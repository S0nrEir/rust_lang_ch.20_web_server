use std::net::TcpListener;
//读写流所需的特定trait
use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use hello::ThreadPool;

static HELLO_HTML_PATH:&str = "html/hello.html";
static NOT_FOUND_HTML_PATH:&str = "html/404.html";
static STATUS_LINE_200:&str = "HTTP/1.1 200 OK";
static STATUS_LINE_404:&str = "HTTP/1.1 404 NOT FOUND";

fn main() {
    //在127.0.0.1:7878上监听tcp流
    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // for stream in listener.incoming() {
    //     //当客户端和服务器发生连接，获取其流，依次处理每个连接
    //     let stream = stream.unwrap();
    //     handle_connection(stream);
    
    //     //handle_connection_simulate_delay(stream);
    // }

    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let pool = ThreadPool::new(4);
    // for stream in listener.incoming(){
    //     let stream = stream.unwrap();
    //     pool.execute(||handle_connection_simulate_delay(stream));
    // }

    //test shutting down:
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection_simulate_delay(stream);
        });
    }

    println!("Shutting down.");
}

///模拟慢速处理
/// 
/// #stream:tcp流
fn handle_connection_simulate_delay(mut stream:TcpStream){
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    //模拟慢五秒处理下一个连接
    let (status_line, filename) = match &request_line[..] {

        "GET / HTTP/1.1" => (STATUS_LINE_200, HELLO_HTML_PATH),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(10));
            (STATUS_LINE_200, HELLO_HTML_PATH)
        }
        _ => (STATUS_LINE_404, NOT_FOUND_HTML_PATH),

    };
    
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

// fn handle_connection(mut stream: TcpStream) {
//     let buf_reader = BufReader::new(&mut stream);
//     let request_line = buf_reader.lines().next().unwrap().unwrap();
//     let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
//         ("HTTP/1.1 200 OK", "html/hello.html")
//     } else {
//         ("HTTP/1.1 404 NOT FOUND", "html/404.html")
//     };

//     let contents = fs::read_to_string(filename).unwrap();
//     let length = contents.len();
//     let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

//     stream.write_all(response.as_bytes()).unwrap();
// }

// fn handle_connection(mut stream:TcpStream){
//     let buf_reader = BufReader::new(&mut stream);
//     //只读第一行，检查是否为一个GET请求
//     let mut response = String::from("");
//     let mut status_line = String::from("");
//     let mut len:usize = 0;
//     let mut contents = String::from("");

//     let request_line = buf_reader.lines().next().unwrap().unwrap();

//     if request_line == "GET / HTTP/1.1"{
//         status_line = "HTTP/1.1 200 OK".to_string();
//         contents = fs::read_to_string("hello.html").unwrap();
//     }else {
//         //不是的话就返回404
//         status_line = "HTTP/1.1 404 NOT FOUND".to_string();
//         contents = fs::read_to_string("404.html").unwrap();
//     }
//     len = contents.len();
//     response = format!("{status_line}\r\nContent-Length:{len}\r\n\r\n{contents}");
//     stream.write_all(response.as_bytes()).unwrap();
//     println!("contents:{}",contents);
// }

//处理连接
// fn handle_connection(mut stream:TcpStream){
//     let buf_reader = BufReader::new(&mut stream);
//     //lines返回每一行的Result<String,std::io::Error>
//     let http_request:Vec<_> = buf_reader.lines()
//         //将每一个result解包
//         .map(|result|result.unwrap())
//         //筛出非空的行
//         .take_while(|line|!line.is_empty())
//         //生成集合返回
//         .collect();

//     //使用 HTTP 1.1 版本的响应，态码为 200，原因短语为 OK，没有 header，也没有 body
//     let status_line = "HTTP/1.1 200 OK";
//     let content = fs::read_to_string("hello.html").unwrap();
//     let len = content.len();
//     let response = format!("{status_line}\r\nContent-Length:{len}\r\n\r\n{content}");

//     //写入连接流
//     stream.write_all(response.as_bytes()).unwrap();
//     println!("Response:{:#?}",response);
// }
