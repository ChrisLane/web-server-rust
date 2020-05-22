extern crate yaml_rust;

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use yaml_rust::YamlLoader;

use web_server::ThreadPool;

fn main() {
    read_config("config.yml");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(format!("{}/{}", "public", filename)).unwrap();
    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn read_config(path: &str) -> Config {
    // Load config file
    let file_str = fs::read_to_string(path).unwrap_or("".to_string());
    let yml = YamlLoader::load_from_str(file_str.as_str()).expect("Failed parsing config");
    let yml = &yml[0];

    println!("Config: {:?}", yml);

    // Handle default port and range checking
    let port = yml["port"].as_i64();
    println!("{:?}", yml["port"]);
    let port_is_valid = port.is_some() && port.unwrap() > 0 && port.unwrap() < u16::MAX as i64;
    let port = if port_is_valid {
        println!("{}", port.unwrap());
        port.unwrap() as u16
    } else {
        eprintln!("Invalid port in config, defaulting to {}", Config::DEFAULT_PORT);
        Config::DEFAULT_PORT
    };

    Config {
        port,
        directory: yml["directory"]
            .as_str()
            .unwrap_or(Config::DEFAULT_PATH.as_ref())
            .parse()
            .unwrap(),
    }
}

#[derive(Debug)]
struct Config {
    port: u16,
    directory: String,
}

impl Config {
    pub const DEFAULT_PORT: u16 = 7878;
    pub const DEFAULT_PATH: &'static str = "public";
}
