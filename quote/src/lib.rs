extern crate irc;
extern crate rand;
extern crate rustc_serialize;

use irc::client::prelude::*;
use irc::error;
use irc::proto::Command::PRIVMSG;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        let resp = if chan == server.config().nickname() {
            user
        } else {
            &chan[..]
        };
        let tokens: Vec<_> = msg.trim_right().split(" ").collect();
        if tokens[0] == "@addquote" && tokens.len() > 1 && msg.len() > 10 + tokens[1].len() {
            let mut quotes = data::Quotes::load(server.config().server());
            let quote = &msg[11+tokens[1].len()..];
            quotes.add_quote(quote, tokens[1]);
            let _ = quotes.save();
            server.send_privmsg(
                resp, &format!("{}: I'll remember it as #{}.", user, quotes.get_latest_index())
            )?;
        } else if tokens[0] == "@quote" {
            let quotes = data::Quotes::load(server.config().server());
            let quote = if tokens.len() > 1 {
                tokens[1].parse().ok().and_then(|i| quotes.get_quote(i))
            } else {
                quotes.get_random_quote()
            };
            match quote {
                Some(q) => server.send_privmsg(resp, &format!("<{}> {}", q.sender, q.message))?,
                None => server.send_privmsg(resp, if tokens.len() > 1 {
                    "There is no such quote."
                } else {
                    "I don't know any quotes."
                })?,
            }
        }
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::fs::{File, create_dir_all};
    use std::io::{Error, ErrorKind, Result};
    use std::io::prelude::*;
    use std::path::Path;
    use rand::{Rng, thread_rng};
    use rustc_serialize::json::{decode, encode};

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Quotes {
        server: String,
        quotes: Vec<Quote>
    }

    impl Quotes {
        pub fn load(server: &str) -> Quotes {
            if let Ok(quotes) = Quotes::load_internal(server) {
                quotes
            } else {
                Quotes { server: server.to_owned(), quotes: Vec::new() }
            }
        }

        fn load_internal(server: &str) -> Result<Quotes> {
            let mut file = try!(File::open(&Path::new(&format!("data/{}.json", server))));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode quotes."
            ))
        }

        pub fn save(&self) -> Result<()> {
            try!(create_dir_all(Path::new("data/")));
            let mut f = try!(File::create(Path::new(&format!("data/{}.json", self.server))));
            try!(f.write_all(try!(encode(self).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode quotes."
            ))).as_bytes()));
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
    use irc::client::conn::MockConnection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
    }

    // TODO: add tests
}
