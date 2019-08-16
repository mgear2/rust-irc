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

/// Returns a TcpStream connected to the desired server, with the given nickname
///
/// # Arguments
///
/// * `nick` - A string that holds the desired user nickname.
/// # `server` - A string that holds the desired irc server
///
/// # Example
///
/// `let stream = connect(nick.to_owned(), server.to_owned()).unwrap();`
///
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

/// Writes a command to a given TcpStream
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TcpStream
/// # `server` - A string that holds the desired message
///
/// # Example
///
/// `send_cmd(&send_stream, "QUIT", "\r\n".to_string())?;`
///
fn send_cmd(mut stream: &TcpStream, cmd: &str, msg: String) -> Result<usize, std::io::Error> {
    let mut cmd = cmd.to_string();
    cmd.push_str(" ");
    cmd.push_str(&msg);
    println!("sending: {}", cmd.trim());
    stream.write(cmd.as_bytes())
}

/// Loops to recieve data from a TcpStream
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TcpStream
///
/// # Example
///
/// The following example demonstrates how to set up a threaded TcpStream with one
/// stream reference listening and one receiving.
/// ```
/// let send_stream = connect(nick.to_owned(), server.to_owned())?;
/// let recv_stream = send_stream.try_clone()?;
/// thread::spawn(move || receive(&recv_stream).expect("error setting up recv_stream"));
/// ```
///
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

/// A client struct holds nickname, server, and command information as well as
/// implementing functions to connect to a server and issue commands.
struct Client {
    /// A user must have a nickname
    nick: String,
    /// A server to connect to must be specified
    server: String,
    /// Storing command data in a hashmap will supply the
    /// user with accurate feedback
    commands: HashMap<String, String>,
}

impl Client {
    /// Returns a Client with the given nickname and server
    /// as well as a hashmap built with command data
    ///
    /// # Arguments
    ///
    /// * `nick` - A string holding the desired nickname
    /// * `server` - A string holding the desired server
    ///
    /// # Example
    ///
    /// `let client = Client::new(nick, server);`
    ///
    fn new(nick: String, server: String) -> Client {
        let mut commands = HashMap::new();
        commands.insert("/quit".to_string(), "Command: /quit".to_string());
        commands.insert(
            "/join".to_string(),
            "Command: /join Parameters: <channel>".to_string(),
        );
        commands.insert(
            "/part".to_string(),
            "Command: /part Parameters: <channel>".to_string(),
        );
        commands.insert(
            "/nick".to_string(),
            "Command: /nick Parameters: <nickname>".to_string(),
        );
        commands.insert(
            "/msg".to_string(),
            "Command: /msg Parameters: <receiver>".to_string(),
        );
        commands.insert(
            "/topic".to_string(),
            "Command: /topic Parameters: <channel> [<topic>]".to_string(),
        );
        commands.insert(
            "/list".to_string(),
            "Command: /list Parameters: <channel>".to_string(),
        );
        commands.insert(
            "/names".to_string(),
            "Command: /names Parameters: <channel>".to_string(),
        );
        Client {
            nick,
            server,
            commands,
        }
    }

    /// Returns an option specifying whether the given
    /// command/message is valid. If the message is not valid,
    /// the function prints information about the command to
    /// the user and returns None.
    ///
    /// # Arguments
    ///
    /// * `params` - The minimum number of params needed for the command
    /// * `msg` - The msg to verify
    ///
    /// # Example
    ///
    /// if let None = self.verify(2, &msg) {
    ///     continue
    /// }
    ///
    fn verify(&self, params: usize, msg: &[&str]) -> Option<()> {
        if msg.len() < params {
            let msg = self.commands.get(msg[0].trim()).unwrap();
            println!("{}", msg);
            return None;
        }
        Some(())
    }

    /// Creates a send and recv TcpStream (with the same socket)
    /// spins recv to its own thread
    /// main thread takes user input and matches it to commands
    /// after commands and processed and messages verified,
    /// the send stream is used to send command/message combinations.
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
                            self.commands
                                .iter()
                                .for_each(|(_, val)| println!("{}", val));
                        }
                        "/quit" => {
                            send_cmd(&send_stream, "QUIT", "\r\n".to_string())?;
                            println!("Quitting...");
                            return Ok(());
                        }
                        "/join" => {
                            if self.verify(2, &msg).is_none() {
                                continue;
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "JOIN", msg)?;
                        }
                        "/part" => {
                            if self.verify(2, &msg).is_none() {
                                continue;
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "PART", msg)?;
                        }
                        "/nick" => {
                            if self.verify(2, &msg).is_none() {
                                continue;
                            }
                            let msg = format!("{}\r\n", msg[1].trim());
                            send_cmd(&send_stream, "NICK", msg)?;
                        }
                        "/msg" => {
                            if self.verify(2, &msg).is_none() {
                                continue;
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
                            let target = if msg.len() > 1 { msg[1].trim() } else { "" };
                            let msg = format!("{}\r\n", target);
                            send_cmd(&send_stream, "LIST", msg)?;
                        }
                        "/names" => {
                            let target = if msg.len() > 1 { msg[1].trim() } else { "" };
                            let msg = format!("{}\r\n", target);
                            send_cmd(&send_stream, "NAMES", msg)?;
                        }
                        "/topic" => {
                            if self.verify(3, &msg).is_none() {
                                continue;
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

/// Parse command line arguments and use them create and run a client
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
