// Copyright © 2019 Matthew Geary
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::process::exit;
use std::str;
use std::thread;

///! A simple IRC client written in Rust.

fn connect(nick: String, mut server: String) -> std::io::Result<TcpStream> {
    server.push_str(":6667");
    // https://doc.rust-lang.org/std/net/struct.TcpStream.html
    let send_stream = TcpStream::connect(server)?;

    // https://tools.ietf.org/html/rfc1459#section-4.1.1
    // https://github.com/hoodie/concatenation_benchmarks-rs
    let nick_string = format!("{}\r\n", &nick);
    let user_string = format!("{} * * {}\n\r", &nick, &nick);

    send_cmd(&send_stream, "USER", user_string)?;
    send_cmd(&send_stream, "NICK", nick_string)?;

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
    stream.write(cmd.as_bytes())
}

fn receive(mut stream: &TcpStream) -> std::io::Result<()> {
    let mut i = 0;
    loop {
        let mut buffer = Vec::new();
        let mut temp = [1];
        for _ in 0..512 {
            stream.read_exact(&mut temp)?;
            match temp[0] {
                0x1 => continue, // start of heading
                0xD => continue, // carriage return
                0xA => break,    // line feed
                _ => buffer.push(temp[0]),
            }
        }
        let res_string = str::from_utf8(&buffer[..]);
        match res_string {
            Ok(_) => {
                if !res_string.unwrap().is_empty() {
                    println!("{}: {}", i, res_string.unwrap());
                    i += 1;
                }
            }
            Err(error) => eprintln!("error while reading from tcp stream: {}", error),
        }
    }
}

struct Client {
    nick: String,
    server: String,
    commands: HashMap<String, String>,
}

impl Client {
    fn new(nick: String, server: String) -> Client {
        let mut commands = HashMap::new();
        commands.insert("/quit".to_string(), "Command: /quit".to_string());
        commands.insert("/join".to_string(), "Command: /join Parameters: <channel>".to_string());
        commands.insert("/part".to_string(), "Command: /part Parameters: <channel>".to_string());
        commands.insert("/nick".to_string(), "Command: /nick Parameters: <nickname>".to_string());
        commands.insert("/msg".to_string(), "Command: /msg Parameters: <receiver>".to_string());
        commands.insert("/topic".to_string(), "Command: /topic Parameters: <channel> [<topic>]".to_string());
        commands.insert("/list".to_string(), "Command: /list Parameters: <channel>".to_string());
        commands.insert("/names".to_string(), "Command: /names Parameters: <channel>".to_string());
        Client {
            nick,
            server,
            commands,
        }
    }

    fn verify(&self, params: usize, msg: &Vec<&str>) -> Option<()> {
        if msg.len() < params {
            let msg = self.commands.get(msg[0].trim()).unwrap();
            println!("{}", msg);
            return Some(());
        }
        None
    }

    fn run(&self) -> std::io::Result<()> {
        let send_stream = connect(self.nick.to_owned(), self.server.to_owned())?;
        let recv_stream = send_stream.try_clone()?;
        // https://doc.rust-lang.org/nightly/std/thread/
        thread::spawn(move || receive(&recv_stream).expect("error setting up recv_stream"));

        // Read the input.
        loop {
            let mut msg = String::new();
            match io::stdin().read_line(&mut msg) {
                Ok(_) => {
                    // https://users.rust-lang.org/t/how-to-split-a-string-by-and-then-print-first-or-last-component/23042
                    let mut msg: Vec<&str> = msg.trim().split(' ').collect();
                    let cmd: &str = msg[0].trim();
                    match cmd {
                        "help" => {
                            self.commands.iter().for_each( |(_, val)| println!("{}", val));
                        }
                        "/quit" => {
                            send_cmd(&send_stream, "QUIT", "\r\n".to_string())?;
                            println!("Quitting...");
                            return Ok(());
                        }
                        "/join" => {
                            if let Some(_) = self.verify(2, &msg) {
                                continue
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "JOIN", msg)?;
                        }
                        "/part" => {
                            if let Some(_) = self.verify(2, &msg) {
                                continue
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "PART", msg)?;
                        }
                        "/nick" => {
                            if let Some(_) = self.verify(2, &msg) {
                                continue
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "NICK", msg)?;
                        }
                        "/msg" => {
                            if let Some(_) = self.verify(2, &msg) {
                                continue
                            }
                            let receiver = msg[1].trim();
                            msg.remove(0);
                            msg.remove(0);
                            let mut text = String::new();
                            msg.iter().for_each(|word| {
                                text.push_str(word);
                                text.push_str(" ")
                            });
                            let text = text.trim();
                            let msg = format!("{} :{:?}\r\n", receiver, text);
                            send_cmd(&send_stream, "PRIVMSG", msg)?;
                        }
                        "/list" => {
                            let mut target = "";
                            if msg.len() > 1 {
                                target = msg[1].trim();
                            }
                            let msg = format!("{}\r\n", target);
                            send_cmd(&send_stream, "LIST", msg)?;
                        }
                        "/names" => {
                            let mut target = "";
                            if msg.len() > 1 {
                                target = msg[1].trim();
                            }
                            let msg = format!("{}\r\n", target);
                            send_cmd(&send_stream, "NAMES", msg)?;
                        }
                        "/topic" => {
                            if let Some(_) = self.verify(3, &msg) {
                                continue
                            }
                            let msg = format!("{} {}\r\n", msg[1].trim(), msg[2].trim());
                            send_cmd(&send_stream, "NAMES", msg)?;
                        }
                        _ => {
                            println!("Unrecognized command: {}", msg[0]);
                        }
                    }
                }
                Err(error) => eprintln!("error while reading user input: {}", error),
            }
        }
    }
}

/// Report proper usage and exit.
fn usage() -> ! {
    eprintln!("rust-irc: usage: rust-irc [username] [server]");
    eprintln!("rust-irc: if no server is supplied, defaults to irc.freenode.net");
    exit(1);
}

/// Do the computation.
fn main() {
    // Process the arguments.
    let args: Vec<String> = std::env::args().collect();
    let mut nick: String;
    let mut server: String;

    match args.len() {
        3 => server = args[2].to_owned(),
        2 => server = "irc.freenode.net".to_string(),
        _ => usage(),
    }

    nick = args[1].to_owned();
    let client = Client::new(nick, server);
    client.run().expect("Client Error");
}

mod tests {
    // extremely important test
    #[test]
    fn extra_testy_test() {
        assert_eq!((2 + 2), 4)
    }
}
