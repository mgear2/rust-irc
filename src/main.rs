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

fn connect(nick: String) -> std::io::Result<TcpStream> {
    let stream = TcpStream::connect("irc.freenode.org:6667")?;

    // https://github.com/hoodie/concatenation_benchmarks-rs
    let nick_string = format!("{}\r\n", &nick);
    let user_string = format!("{} * * {}\n\r", &nick, &nick);

    let _ = send_cmd(&stream, "USER", user_string);
    let _ = send_cmd(&stream, "NICK", nick_string);

    Ok(stream)
}

fn _send_msg(mut stream: &TcpStream, msg: String) -> Result<usize, std::io::Error> {
    let mut priv_msg = "PRIVMSG ".to_string();
    priv_msg.push_str(&msg);
    stream.write(msg.as_bytes())
}

fn send_cmd(mut stream: &TcpStream, cmd: &str, msg: String) -> Result<usize, std::io::Error> {
    let mut cmd = cmd.to_string();
    cmd.push_str(" ");
    cmd.push_str(&msg);
    stream.write(cmd.as_bytes())
}

fn _receive() {}

fn _print_response() {}

/// Do the computation.
fn main() -> std::io::Result<()> {
    // Process the arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }
    let nick = args[1].clone();
    let _channel = &args[2];
    let mut stream = connect(nick).unwrap();

    for _ in 0..76 {
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
        println!("res_string: {}", res_string);
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
