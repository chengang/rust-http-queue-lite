extern crate rustc_serialize;
extern crate docopt;
extern crate threadpool;

use docopt::Docopt;
use threadpool::ThreadPool;

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Arc;
use std::net::Ipv4Addr;

static USAGE: &'static str = "
Usage: http-queue-lite <ip> <port>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_ip: String,
    arg_port: u16,
}


#[allow(dead_code)]
struct RemoteAddr {
    ip: String,
    port: u16,
}

#[allow(dead_code)]
struct RequestLine {
    method: String,
    request_uri: String,
    protocol_version: String,
    request_script: String,
    query_string: String,
    get_argv: HashMap<String, String>,
}

#[allow(dead_code)]
struct RequestHeader {
    user_agent: String,
    host: String,
    accept: String,
}

#[allow(dead_code)]
struct RequestInfo {
    remote_ip: String,
    remote_port: u16,
    method: String,
    request_uri: String,
    protocol_version: String,
    request_script: String,
    query_string: String,
    get_argv: HashMap<String, String>,
    header: RequestHeader,
}

// done
fn get_remote_addr(stream: &TcpStream) -> RemoteAddr {
    let peer = stream.peer_addr().unwrap().to_string();
    let v: Vec<&str> = peer.split(':').collect();

    let peer_ip = v[0].to_string();
    let peer_port = v[1].parse::<u16>().ok().expect("fail parse port to i32");

    let remote_addr = RemoteAddr {ip: peer_ip, port: peer_port};
    return remote_addr;
}

fn ht_readline(mut stream: &TcpStream) -> String {
    let mut result = String::new();
    loop {
        let mut buf = [0u8];
        let _ = stream.read(&mut buf);
        if buf[0]==13 { break; }
        if buf[0]==10 { continue; }
        result.push(buf[0] as char);
    }
    //println!("[{}]", result);
    return result;
}

#[allow(unused_assignments)]
fn get_request_line(stream: &TcpStream) -> RequestLine {
    let line = ht_readline(&stream);
    let v: Vec<&str> = line.split(' ').collect();
    let method = v[0].to_string();
    let request_uri = v[1].to_string();
    let protocol_version = v[2].to_string();

    let mut request_script = String::new();
    let mut query_string = String::new();
    let mut get_argv = HashMap::new();
    {
        let v2: Vec<&str> = request_uri.split('?').collect();
        request_script = v2[0].to_string();
        if v2.len() > 1 { 
            query_string = v2[1].to_string();
            let v3: Vec<&str> = v2[1].split('&').collect();
            for kv_pair in v3.iter() {
                let v4: Vec<&str> = kv_pair.split('=').collect();
                if v4.len() > 1 {
                    get_argv.insert(v4[0].to_string(), v4[1].to_string());
                } else {
                    get_argv.insert(v4[0].to_string(), "".to_string());
                }
            }
        }
    }

    let request_line = RequestLine {
        method: method, 
            request_uri: request_uri, 
            protocol_version: protocol_version,
            request_script: request_script,
            query_string: query_string,
            get_argv: get_argv,
    };
    return request_line;
}

fn get_request_header(stream: &TcpStream) -> RequestHeader {
    let mut request_header = RequestHeader {
        user_agent: "".to_string(),
        host: "".to_string(),
        accept: "".to_string(),
    };
    loop {
        let line = ht_readline(&stream);
        if line.is_empty() { break; }
        let v: Vec<&str> = line.split(' ').collect();
        match v[0] {
            "User-Agent:" => { request_header.user_agent = v[1].to_string(); },
            "Host:" => { request_header.host = v[1].to_string(); },
            "Accept:" => { request_header.accept = v[1].to_string(); },
            _ => {},
        }
    }

    return request_header;
}

fn get_request_info(stream: &TcpStream) -> RequestInfo {
    let request_line = get_request_line(&stream);
    let request_header = get_request_header(&stream);
    let remote_addr = get_remote_addr(&stream);
    let request_info = RequestInfo {
        remote_ip: remote_addr.ip,
        remote_port: remote_addr.port,
        method: request_line.method,
        request_uri: request_line.request_uri,
        request_script: request_line.request_script,
        query_string: request_line.query_string,
        protocol_version: request_line.protocol_version,
        get_argv: request_line.get_argv,
        header: request_header,
    };
    return request_info;
}

fn handle_client(mut stream: TcpStream, mut tasks: MutexGuard<Vec<String>>) {
    let request_info = get_request_info(&stream);

    let mut body = String::new();
    let mut status = String::new();
    if request_info.request_script.contains("add") {
        status.push_str("200 OK");
        if tasks.len() < 10000000 {
            tasks.insert(0, request_info.query_string);
            body.push_str("added ok");
        } else {
            body.push_str("queue full");
        }
    } else if request_info.request_script.contains("get") {
        status.push_str("200 OK");
        if tasks.len() > 0 {
            let task = tasks.pop().unwrap();
            body = format!("{}", task);
        } else {
            body.push_str("queue empty");
        }
    } else {
        status.push_str("404 Not Found");
        body.push_str("Not Found");
    }
    let response = format!("HTTP/1.0 {}\r\n\
                       Server: HTTPQ\r\n\
                       Content-Length: {}\r\n\
                       \r\n\
                       {}", 
                       status, body.len(), body);
    let _ =  stream.write(response.as_bytes());
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    let listener_ip = args.arg_ip.parse::<Ipv4Addr>().unwrap();

    let listener = TcpListener::bind((listener_ip, args.arg_port)).unwrap();
    let tasks: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let pool = ThreadPool::new(32);

    println!("HTTP Queue Lite Started.");
    for stream in listener.incoming() {
        let tasks = tasks.clone();
        match stream {
            Ok(stream) => {
                pool.execute(move|| {
                    let tasks = tasks.lock().unwrap();
                    handle_client(stream, tasks);
                });
            }
            Err(e) => { let _ = e;}
        }
    }
    drop(listener);
}
