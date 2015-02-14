#![feature(collections, core, io, path, slicing_syntax, std_misc)]
extern crate irc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;


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
        let mut messages = data::Messages::load();
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if tokens[0] == "@tell" && tokens.len() > 1 && tokens[1] != server.config().nickname()
        && msg.len() > 7 + tokens[1].len() {
            let message = &msg[7+tokens[1].len()..];
            messages.add_message(tokens[1], message, user);
            let _ = messages.save();
            try!(server.send_privmsg(resp, &format!("{}: I'll let them know!", user)[]));
        } else if tokens[0] == "@tell" && tokens.len() > 1 
               && tokens[1] == server.config().nickname() {
            try!(server.send_privmsg(resp, &format!("{}: I'm right here!", user)[]));
        }
        for msg in messages.get_messages(user).iter() {
            try!(server.send_privmsg(resp, &msg.to_string()[]));
        }
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    use std::error::Error;
    use std::string::ToString;
    use std::old_io::{File, FilePermission, InvalidInput, IoError, IoResult};
    use std::old_io::fs::mkdir_recursive;
    use rustc_serialize::json::{decode, encode};
    use time::{Timespec, get_time};

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Messages {
        undelivered: HashMap<String, Vec<Message>>
    }

    impl Messages {
        pub fn load() -> Messages {
            if let Ok(messages) = Messages::load_internal() {
                messages
            } else {
                Messages { undelivered: HashMap::new() }
            }
        }

        fn load_internal() -> IoResult<Messages> {
            let mut file = try!(File::open(&Path::new("data/messages.json")));
            let data = try!(file.read_to_string());
            decode(&data[]).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to decode messages.",
                detail: Some(e.description().to_owned()),
            })
        }

        pub fn save(&self) -> IoResult<()> {
            try!(mkdir_recursive(&Path::new("data/"), FilePermission::all()));
            let mut f = File::create(&Path::new("data/messages.json"));
            f.write_str(&try!(encode(self).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Failed to encode messages.",
                detail: Some(e.description().to_owned()),
            }))[])
        }

        pub fn add_message(&mut self, target: &str, message: &str, sender: &str) {
            match self.undelivered.entry(target.to_owned()) {
                Occupied(mut e) => e.get_mut().push(Message::new(target, message, sender)),
                Vacant(e) => { e.insert(vec![Message::new(target, message, sender)]); },
            }
        }

        pub fn get_messages(&mut self, user: &str) -> Vec<Message> {
            let ret = match self.undelivered.remove(user) {
                Some(v) => v,
                None => vec![],
            };
            let _ = self.save();
            ret
        }
    }

    #[derive(Clone, RustcDecodable, RustcEncodable)]
    struct Message {
        target: String,
        sender: String,
        message: String,
        time: Timespec,
    }

    impl Message {
        pub fn new(target: &str, message: &str, sender: &str) -> Message {
            Message {
                target: target.to_owned(),
                sender: sender.to_owned(),
                message: message.to_owned(),
                time: get_time(),
            }
        }
    }

    impl ToString for Message {
        fn to_string(&self) -> String {
            let dur = get_time() - self.time;
            let ago = if dur.num_weeks() > 1 {
                format!("{} weeks ago", dur.num_weeks())
            } else if dur.num_weeks() == 1 {
                "A week ago".to_owned()
            } else if dur.num_days() > 1 {
                format!("{} days ago", dur.num_days())
            } else if dur.num_days() == 1 {
                "A day ago".to_owned()
            } else if dur.num_hours() > 1 {
                format!("{} hours ago", dur.num_hours())
            } else if dur.num_hours() == 1 {
                "An hour ago".to_owned()
            } else if dur.num_minutes() > 1 {
                format!("{} minutes ago", dur.num_minutes())
            } else if dur.num_minutes() == 1 {
                "A minute ago".to_owned()
            } else {
                "Moments ago".to_owned()  
            };
            format!("{}: {}, {} said {}{}", self.target, ago, self.sender, self.message,
                if self.message.ends_with(".") || self.message.ends_with("!") || 
                self.message.ends_with("?") { "" } else { "." })
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
