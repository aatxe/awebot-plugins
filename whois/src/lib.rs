#![feature(io)]
extern crate irc;
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
        if msg.starts_with("@iam ") {
            let me = data::WhoIs::new(user, &msg[5..]);
            let msg = match me.save() {
                Ok(_) => format!("{}: Got it!", user),
                Err(_) => format!("{}: Something went wrong.", user),
            };
            try!(server.send_privmsg(resp, &msg));
        } else if msg.starts_with("@whois ") {
            let tokens: Vec<_> = msg.split(" ").collect();
            let msg = match data::WhoIs::load(tokens[1]) {
                Ok(whois) => format!("{}: {} is {}", user, whois.nickname, whois.description),
                Err(_) => format!("{}: I don't know who {} is.", user, tokens[1]),
            };
            try!(server.send_privmsg(resp, &msg));
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
    use rustc_serialize::json::{decode, encode};
    
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct WhoIs {
        pub nickname: String,
        pub description: String,
    }

    impl WhoIs {
        pub fn new(nickname: &str, description: &str) -> WhoIs {
            WhoIs { nickname: nickname.to_owned(), description: description.to_owned() }
        }

        pub fn load(nickname: &str) -> Result<WhoIs> {
            let mut path = "data/whois/".to_owned();
            path.push_str(nickname);
            path.push_str(".json");
            let mut file = try!(File::open(Path::new(&path)));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|e| 
                 Error::new(ErrorKind::InvalidInput, "Failed to decode whois data.",
                           Some(e.description().to_owned()))
            )
        }

        pub fn save(&self) -> Result<()> {
            let mut path = "data/whois/".to_owned();
            try!(create_dir_all(Path::new(&path)));
            path.push_str(&self.nickname);
            path.push_str(".json");
            let mut f = try!(File::create(&Path::new(&path)));
            try!(f.write_all(try!(encode(self).map_err(|e| 
                Error::new(ErrorKind::InvalidInput, "Failed to encode whois data.",
                           Some(e.description().to_owned()))
            )).as_bytes()));
            f.flush()
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
