use std::result;
use std::thread;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;

//id池被多个线程访问，考虑数据竞争的问题，使用原子操作
///线程id池
static ID_POOL:AtomicUsize = AtomicUsize::new(0);

///worker持有的线程任务回调
/// 
/// #Job:任务回调类型
type Job = Box<dyn FnOnce() + Send + 'static>;

///线程池
/// 
/// #_workers:线程池中的worker集合
/// 
/// #panic:没有足够的系统资源
pub struct ThreadPool{
    _workers : Vec<Worker>,
    _sender : mpsc::Sender<Job>,
}

///线程管理类，每一个实例对应一个thread
///用于处理线程的开启/关闭
/// 
/// #_id:线程id
/// 
/// #_thread:线程句柄
struct Worker{
    _id:usize,
    _thread:thread::JoinHandle<()>,
    // _receiver:mpsc::Receiver<Job>,
    // _receiver:Arc<Mutex<mpsc::Receiver<Job>>>,
}

impl ThreadPool {
    ///创建线程池
    /// #panic!:if size equals 0
    pub fn new(size:usize) -> ThreadPool{
        assert!(size > 0);
        let (sender,receiver) = mpsc::channel();
        //将receiver包装成一个Arc<Mutex<mpsc::Receiver<Job>>>
        //对于每个worker，克隆Arc获取其引用以共享所有权
        let recevier = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size{
            workers.push(Worker::new(Arc::clone(&recevier)));
        }
        
        return ThreadPool{
            _workers:workers,
            _sender:sender,
        };
    }

    ///execute希望执行一个返回值为void的闭包
    pub fn execute<F>(&self,f:F)
    where
    //FnOnce() trait代表闭包不接受参数，2
    //Send trait代表闭包可以在线程间传递，
    //'static代表闭包不引用任何外部变量(因为不知道闭包要跑多久，所以使用全局的)
        F : FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        //通过信道发送方将job传给接收方来执行线程任务
        self._sender.send(job).unwrap();
    }
}

impl Worker {
    pub fn new(receiver:Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker{
        let id = Self::gen_id();
        println!("Worker {} created.",id);
        let thread = thread::spawn(move || {

            //当先传入一个阻塞线程的job，然后传入一个立即执行的job，第一种情况会并行处理job，而第二种情况不会，这是为什么？
            //示例 20-20 中的代码使用的 let job = receiver.lock().unwrap().recv().unwrap(); 
            //之所以可以工作是因为对于 let 来说，当 let 语句结束时任何表达式中等号右侧使用的临时值都会立即被丢弃。
            //然而 while let（if let 和 match）直到相关的代码块结束都不会丢弃临时值。
            //在示例 20-21 中，job() 调用期间锁一直持续，这也意味着其他的 worker 无法接受任务。

            loop {
                //阻塞线程，直到有job可用
                //如果没有可用的job，线程将一直停留在这里，直到有job可用。
                
                println!("Worker {} waiting for a job.",id);
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.",id);
                job();
                println!("Worker {} finish a job.",id);

                //不阻塞线程，如果拿不到job，就立即执行下一次循环
                // if  let Ok(result) = receiver.lock(){
                //     if let Ok(job) = result.recv(){
                //         println!("Worker {} got a job; executing.",id);
                //         job();
                //         println!("Worker {} finish a job.",id);
                //     }else {
                //         println!("Worker {} faild to get job.",id);
                //     }
                // }else {
                //     println!("Worker {} faild to get result.",id);
                // }
            }
        });

        return Worker{
            _id:id,
            _thread:thread,
        };
    }

    ///从id池生成一个id
    fn gen_id() -> usize{
        return ID_POOL.fetch_add(1, Ordering::SeqCst);
    }
}