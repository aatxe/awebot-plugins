#![feature(collections, core, old_io, old_path)]
extern crate irc;
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
        if msg.starts_with("@iam ") {
            let me = data::WhoIs::new(user, &msg[5..]);
            let msg = match me.save() {
                Ok(_) => format!("{}: Got it!", user),
                Err(_) => format!("{}: Something went wrong.", user),
            };
            try!(server.send_privmsg(resp, &msg));
        } else if msg.starts_with("@whois ") {
            let tokens: Vec<_> = msg.split_str(" ").collect();
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
    use std::error::Error;
    use std::old_io::fs::{File, mkdir_recursive};
    use std::old_io::{FilePermission, InvalidInput, IoError, IoResult};
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

        pub fn load(nickname: &str) -> IoResult<WhoIs> {
            let mut path = "data/whois/".to_owned();
            path.push_str(nickname);
            path.push_str(".json");
            let mut file = try!(File::open(&Path::new(&path)));
            let data = try!(file.read_to_string());
            decode(&data).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to decode whois data.",
                detail: Some(e.description().to_owned()),
            })
        }

        pub fn save(&self) -> IoResult<()> {
            let mut path = "data/whois/".to_owned();
            try!(mkdir_recursive(&Path::new(&path), FilePermission::all()));
            path.push_str(&self.nickname);
            path.push_str(".json");
            let mut f = File::create(&Path::new(&path));
            f.write_str(&try!(encode(self).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to encode whois data.",
                detail: Some(e.description().to_owned()),
            }))[..])
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
        let vec = server.conn().writer().get_ref().to_vec();
        String::from_utf8(vec).unwrap()
    }
    
    // TODO: add tests
}
