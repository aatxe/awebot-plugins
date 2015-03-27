#![feature(io)]
extern crate irc;
extern crate rand;
extern crate rustc_serialize;

use std::io::{BufReader, BufWriter, Result};
use irc::client::conn::NetStream;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>, 
                   message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a ServerExt<'a, T, U>, msg: &Message) -> Result<()> 
    where T: IrcRead, U: IrcWrite {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        let resp = if chan == server.config().nickname() {
            user
        } else {
            &chan[..]
        };
        let tokens: Vec<_> = msg.split(" ").collect();
        if tokens[0] == "@addquote" && tokens.len() > 1 && msg.len() > 10 + tokens[1].len() {
            let mut quotes = data::Quotes::load();
            let quote = &msg[11+tokens[1].len()..];
            quotes.add_quote(quote, tokens[1]);
            let _ = quotes.save();
            try!(server.send_privmsg(resp, &format!("{}: I'll remember it as #{}.", user, 
                                                    quotes.get_latest_index())));
        } else if tokens[0] == "@quote" {
            let quotes = data::Quotes::load();
            let quote = if tokens.len() > 1 {
                tokens[1].parse().ok().and_then(|i| quotes.get_quote(i))
            } else {
                quotes.get_random_quote()
            };
            match quote {
                Some(q) => 
                    try!(server.send_privmsg(resp, &format!("<{}> {}", q.sender, q.message))),
                None => try!(server.send_privmsg(resp, if tokens.len() > 1 {
                    "There is no such quote."
                } else {
                    "I don't know any quotes."
                })),
            }
        }
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::error::Error as StdError;
    use std::fs::{File, create_dir_all};
    use std::io::{Error, ErrorKind, Result};
    use std::io::prelude::*;
    use std::path::Path;
    use rand::{Rng, thread_rng};
    use rustc_serialize::json::{decode, encode};

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Quotes {
        quotes: Vec<Quote>
    }

    impl Quotes {
        pub fn load() -> Quotes {
            if let Ok(quotes) = Quotes::load_internal() {
                quotes
            } else {
                Quotes { quotes: Vec::new() }
            }
        }

        fn load_internal() -> Result<Quotes> {
            let mut file = try!(File::open(&Path::new("data/quotes.json")));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|e| 
                Error::new(ErrorKind::InvalidInput, "Failed to decode quotes.",
                           Some(e.description().to_owned()))
            )
        }

        pub fn save(&self) -> Result<()> {
            try!(create_dir_all(Path::new("data/")));
            let mut f = try!(File::create(Path::new("data/quotes.json")));
            try!(f.write_all(try!(encode(self).map_err(|e| 
                Error::new(ErrorKind::InvalidInput, "Failed to decode quotes.",
                           Some(e.description().to_owned()))
            )).as_bytes()));
            f.flush()
        }

        pub fn add_quote(&mut self, message: &str, sender: &str) {
            self.quotes.push(Quote::new(message, sender));
        }

        pub fn get_quote(&self, index: usize) -> Option<&Quote> {
            self.quotes.get(index - 1)
        }

        pub fn get_random_quote(&self) -> Option<&Quote> {
            thread_rng().choose(&self.quotes)
        }

        pub fn get_latest_index(&self) -> usize {
            self.quotes.len()
        }
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Quote {
        pub message: String,
        pub sender: String,
    }

    impl Quote {
        pub fn new(message: &str, sender: &str) -> Quote {
            Quote { message: message.to_owned(), sender: sender.to_owned() }
        }
    }
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            Cursor::new(input.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        let vec = server.conn().writer().to_vec();
        String::from_utf8(vec).unwrap() 
    }
    
    // TODO: add tests
}
