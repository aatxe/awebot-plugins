#![feature(collections, core, io, path, slicing_syntax)]
extern crate irc;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        let resp = if chan == server.config().nickname() {
            user
        } else {
            chan
        };
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if tokens[0] == "@addquote" && tokens.len() > 1 && msg.len() > 10 + tokens[1].len() {
            let mut quotes = data::Quotes::load();
            let quote = &msg[11+tokens[1].len()..];
            quotes.add_quote(quote, tokens[1]);
            let _ = quotes.save();
            try!(server.send_privmsg(resp, &format!("{}: I'll remember it as #{}.", user, 
                                                    quotes.get_latest_index())[]));
        } else if tokens[0] == "@quote" {
            let quotes = data::Quotes::load();
            let quote = if tokens.len() > 1 {
                tokens[1].parse().ok().and_then(|i| quotes.get_quote(i))
            } else {
                quotes.get_random_quote()
            };
            match quote {
                Some(q) => 
                    try!(server.send_privmsg(resp, &format!("<{}> {}", q.sender, q.message)[])),
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
    use std::error::Error;
    use std::old_io::{File, FilePermission, InvalidInput, IoError, IoResult};
    use std::old_io::fs::mkdir_recursive;
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

        fn load_internal() -> IoResult<Quotes> {
            let mut file = try!(File::open(&Path::new("data/quotes.json")));
            let data = try!(file.read_to_string());
            decode(&data[]).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to decode quotes.",
                detail: Some(e.description().to_owned()),
            })
        }

        pub fn save(&self) -> IoResult<()> {
            try!(mkdir_recursive(&Path::new("data/"), FilePermission::all()));
            let mut f = File::create(&Path::new("data/quotes.json"));
            f.write_str(&try!(encode(self).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to encode quotes.",
                detail: Some(e.description().to_owned()),
            }))[])
        }

        pub fn add_quote(&mut self, message: &str, sender: &str) {
            self.quotes.push(Quote::new(message, sender));
        }

        pub fn get_quote(&self, index: usize) -> Option<&Quote> {
            self.quotes.get(index - 1)
        }

        pub fn get_random_quote(&self) -> Option<&Quote> {
            thread_rng().choose(&self.quotes[])
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
    use std::old_io::{MemReader, MemWriter};
    use irc::client::conn::Connection;
    use irc::client::server::{IrcServer, Server};
    use irc::client::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&Wrapper::new(&server), &message).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }
    
    // TODO: add tests
}
