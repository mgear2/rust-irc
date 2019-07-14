// Copyright © 2019 Matthew Geary
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use std::process::exit;

///! A simple IRC client written in Rust.

/// Report proper usage and exit.
fn usage() -> ! {
    eprintln!("rust-irc: usage: rust-irc [username] [channel]");
    exit(1);
}

/// Send a message
fn send(msg: String) {
    println!("Sent: {}", msg);
}

/// Do the computation.
fn main() {
    // Process the arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }
    let username = &args[1];
    let channel = &args[2];

    println!("username: {}, channel: {}", username, channel);

    // Read the input.
    use std::io;
    loop {
        let mut msg = String::new();
        match io::stdin().read_line(&mut msg) {
            Ok(n) => {
                msg = msg.trim().to_string();
                println!("{} bytes read", n);
                if msg == "exit" {
                    println!("exiting...");
                    break;
                } else {
                    send(msg);
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
