extern crate irc;
extern crate rustc_serialize;

use irc::client::prelude::*;
use irc::error;
use irc::error::Command::PRIVMSG;

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
        if msg.starts_with("@iam ") {
            let me = data::WhoIs::new(user, &msg[5..]);
            let msg = match me.save() {
                Ok(_) => format!("{}: Got it!", user),
                Err(_) => format!("{}: Something went wrong.", user),
            };
            server.send_privmsg(resp, &msg)?;
        } else if tokens[0] == "@whois" || tokens[0] == "@whodat" {
            let msg = if tokens.len() > 1 {
                match data::WhoIs::load(tokens[1]) {
                    Ok(whois) => format!("{}: {} is {}", user, whois.nickname, whois.description),
                    Err(_) => format!("{}: I don't know who {} is.", user, tokens[1]),
                }
            } else {
                format!("{}: Who is who? I need a name!", user)
            };
            server.send_privmsg(resp, &msg)?;
        } else if tokens[0] == "@whoami" {
            let msg = match data::WhoIs::load(user) {
                Ok(whois) => format!("{}: you are {}", user, whois.description),
                Err(_) => format!("{}: I don't know who you are.", user),
            };
            server.send_privmsg(resp, &msg)?;
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
    use rustc_serialize::json::{decode, encode};

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct WhoIs {
        pub nickname: String,
        pub description: String,
    }

    impl WhoIs {
        pub fn new(nickname: &str, description: &str) -> WhoIs {
            WhoIs { nickname: nickname.to_lowercase(), description: description.to_owned() }
        }

        pub fn load(nickname: &str) -> Result<WhoIs> {
            let mut path = "data/whois/".to_owned();
            path.push_str(&nickname.to_lowercase());
            path.push_str(".json");
            let mut file = try!(File::open(Path::new(&path)));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode whois data."
            ))
        }

        pub fn save(&self) -> Result<()> {
            let mut path = "data/whois/".to_owned();
            try!(create_dir_all(Path::new(&path)));
            path.push_str(&self.nickname);
            path.push_str(".json");
            let mut f = try!(File::create(&Path::new(&path)));
            try!(f.write_all(try!(encode(self).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to encode whois data."
            ))).as_bytes()));
            f.flush()
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
