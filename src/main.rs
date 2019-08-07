// Copyright © 2019 Matthew Geary
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use std::io::prelude::*;
use std::net::TcpStream;
use std::process::exit;
use std::str;
use std::thread;
// use std::time::Duration;

///! A simple IRC client written in Rust.

/// Report proper usage and exit.
fn usage() -> ! {
    eprintln!("rust-irc: usage: rust-irc [username] [channel]");
    exit(1);
}

fn connect(nick: String) -> std::io::Result<TcpStream> {
    // https://doc.rust-lang.org/std/net/struct.TcpStream.html
    let send_stream = TcpStream::connect("irc.freenode.net:6667")?;

    // https://tools.ietf.org/html/rfc1459#section-4.1.1
    // https://github.com/hoodie/concatenation_benchmarks-rs
    let nick_string = format!("{}\r\n", &nick);
    let user_string = format!("{} * * {}\n\r", &nick, &nick);

    let _ = send_cmd(&send_stream, "USER", user_string);
    let _ = send_cmd(&send_stream, "NICK", nick_string);

    Ok(send_stream)
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
    println!("sending: {}", cmd.trim());
    let res = stream.write(cmd.as_bytes());
    println!("{:?}", res);
    return res
}

fn receive(mut stream: &TcpStream) {
    let mut i = 0;
    loop {
        let mut buffer = Vec::new();
        let mut temp = [1];
        for _ in 0..512 {
            let _ = stream.read_exact(&mut temp);
            match temp[0] {
                0x1 => continue, // start of heading
                0xD => continue, // carriage return
                0xA => break,    // line feed
                _ => buffer.push(temp[0]),
            }
        }
        let res_string = str::from_utf8(&buffer[..]).unwrap();
        if !res_string.is_empty() {
            println!("{}: {}", i, res_string);
            i += 1;
        }
    }
}

/// Do the computation.
fn main() -> std::io::Result<()> {
    // Process the arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }
    let nick = args[1].clone();
    let _channel = &args[2];
    let send_stream = connect(nick)?;
    let recv_stream = send_stream.try_clone()?;

    // https://doc.rust-lang.org/nightly/std/thread/
    thread::spawn( move || {
        let _ = receive(&recv_stream);
    });

    // Read the input.
    use std::io;
    loop {
        let mut msg = String::new();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                // https://users.rust-lang.org/t/how-to-split-a-string-by-and-then-print-first-or-last-component/23042
                let msg: Vec<&str> = msg.trim().split(" ").collect();
                let cmd: &str = msg[0].trim();
                match cmd {
                    "/quit" => {
                        let _ = send_cmd(&send_stream, "QUIT", "\r\n".to_string());
                        println!("Quitting...");
                        return Ok(());
                    },
                    "/join" => {
                        let join_msg = format!("{}\r\n", msg[1].trim());
                        let _ = send_cmd(&send_stream, "JOIN", join_msg);
                    }
                    _ => {
                        println!("Unrecognized command: {}", msg[0]);
                    }
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
