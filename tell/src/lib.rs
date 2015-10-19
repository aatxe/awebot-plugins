extern crate irc;
extern crate rustc_serialize;
extern crate time;


use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;

#[no_mangle]
pub extern fn process(server: &NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = msg.into() {
        let resp = if chan == server.config().nickname() {
            user
        } else {
            &chan[..]
        };
        let mut messages = data::Messages::load(server.config().server());
        let tokens: Vec<_> = msg.trim_right().split(" ").collect();
        if tokens[0] == "@tell" && tokens.len() > 1 && tokens[1] != server.config().nickname()
        && msg.len() > 7 + tokens[1].len() {
            if messages.is_recent(user, tokens[1]) {
                try!(server.send_privmsg(resp,
                    &format!("{}: You've sent {} a message too recently! Wait a minute!", user,
                             tokens[1])
                ));
            } else {
                let message = &msg[7+tokens[1].len()..];
                messages.add_message(tokens[1], message, user);
                let _ = messages.save();
                try!(server.send_privmsg(resp, &format!("{}: I'll let them know!", user)));
            }
        } else if tokens[0] == "@tell" && tokens.len() > 1
               && tokens[1] == server.config().nickname() {
            try!(server.send_privmsg(resp, &format!("{}: I'm right here!", user)));
        }
        for msg in messages.get_messages(user).iter() {
            try!(server.send_privmsg(resp, &msg.to_string()));
        }
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    use std::fs::{File, create_dir_all};
    use std::io::{Error, ErrorKind, Result};
    use std::io::prelude::*;
    use std::path::Path;
    use std::string::ToString;
    use rustc_serialize::json::{decode, encode};
    use time::{Timespec, get_time};

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Messages {
        server: String,
        undelivered: HashMap<String, Vec<Message>>
    }

    impl Messages {
        pub fn load(server: &str) -> Messages {
            if let Ok(messages) = Messages::load_internal(server) {
                messages
            } else {
                Messages {
                    server: server.to_owned(),
                    undelivered: HashMap::new()
                }
            }
        }

        fn load_internal(server: &str) -> Result<Messages> {
            let mut file = try!(File::open(&Path::new(&format!("data/{}.json", server))));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode messages."
            ))
        }

        pub fn save(&self) -> Result<()> {
            try!(create_dir_all(&Path::new("data/")));
            let mut f = try!(File::create(&Path::new(&format!("data/{}.json", self.server))));
            f.write_all(&try!(encode(self).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to encode messages."
            ))).as_bytes())
        }

        pub fn is_recent(&self, from: &str, to: &str) -> bool {
            if let Some(msg) = self.undelivered.get(&to.to_lowercase()).and_then(|v| v.last()) {
                &msg.sender[..] == from && (get_time() - msg.time).num_minutes() < 1
            } else {
                false
            }
        }

        pub fn add_message(&mut self, target: &str, message: &str, sender: &str) {
            match self.undelivered.entry(target.to_lowercase()) {
                Occupied(mut e) => e.get_mut().push(Message::new(target, message, sender)),
                Vacant(e) => { e.insert(vec![Message::new(target, message, sender)]); },
            }
        }

        pub fn get_messages(&mut self, user: &str) -> Vec<Message> {
            let ret = match self.undelivered.remove(user.to_lowercase()) {
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
