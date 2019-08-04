// Copyright © 2019 Matthew Geary
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use std::io::prelude::*;
use std::net::TcpStream;
use std::process::exit;
use std::str;

///! A simple IRC client written in Rust.

/// Report proper usage and exit.
fn usage() -> ! {
    eprintln!("rust-irc: usage: rust-irc [username] [channel]");
    exit(1);
}

fn _send_msg() {}

fn _send_cmd() {}

fn _receive() {}

fn _print_response() {}

/// Do the computation.
fn main() -> std::io::Result<()> {
    // Process the arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }
    let mut username = args[1].clone();
    let channel = &args[2];
    println!("username: {:?}, channel: {}", username, channel);
    username.push_str(&"\r\n".to_string());
    println!("username: {:?}, channel: {}", username, channel);
    let mut stream = TcpStream::connect("irc.freenode.org:6667")?;
    let _ = stream.write(username.as_bytes())?;
    //let _ = stream.write(b"USER MATT5\r\n");
    //let _ = stream.write(b"JOIN #tutbot-testing\r\n");

    for _ in 0..10 {
        let mut buffer = Vec::new();
        let mut temp = [1];
        for _ in 0..512 {
            stream.read_exact(&mut temp)?;
            if temp[0] == 13 {
                break;
            }
            buffer.push(temp[0]);
        }
        let res_string = str::from_utf8(&buffer[..]).unwrap();
        println!("result: , buffer: {:?}, res_string: {}", buffer, res_string);
    }

    // Read the input.
    use std::io;
    loop {
        let mut msg = String::new();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                msg = msg.trim().to_string();
                if msg == "exit" {
                    println!("exiting...");
                    return Ok(());
                } else {
                    //let recv = stream.write(&msg.as_bytes());
                    //println!("Recv: {:?}", recv);
                    // https://codereview.stackexchange.com/questions/110073/simple-tcp-client-in-rust
                    //let mut buffer = String::new();
                    //let result = stream.read_to_string(&mut buffer);
                    //println!("result: {:?}, buffer: {}", result, buffer);
                }
            }
            Err(error) => eprintln!("error: {}", error),
        }
    }
}

mod tests {
    // extremely important test
    #[test]
    fn extra_testy_test() {
        assert_eq!((2 + 2), 4)
    }
}
