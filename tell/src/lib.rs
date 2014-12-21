#![feature(slicing_syntax)]
extern crate irc;
extern crate serialize;
extern crate time;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    let mut args = Vec::new();
    let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
    args.push_all(msg_args[]);
    if let Some(ref suffix) = message.suffix {
        args.push(suffix[])
    }
    let source = message.prefix.unwrap_or(String::new());
    process_internal(server, source[], message.command[], args[])
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, source: &str, command: &str, 
                                  args: &[&str]) -> IoResult<()> where T: IrcReader, U: IrcWriter {
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        let mut messages = data::Messages::load();
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if tokens[0] == "@tell" {
            let message = msg[7+tokens[1].len()..];
            messages.add_message(tokens[1], message, user);
            let _ = messages.save();
            try!(server.send_privmsg(chan, "I'll do it when I see them!"));
        }
        for msg in messages.get_messages(user).iter() {
            try!(server.send_privmsg(chan, msg.to_string()[]));
        }
    }
    Ok(())
}

mod data {
    use std::string::ToString;
    use std::io::{File, FilePermission, InvalidInput, IoError, IoResult};
    use std::io::fs::mkdir_recursive;
    use serialize::json::{decode, encode};
    use time::{Timespec, get_time};

    #[deriving(Encodable, Decodable)]
    pub struct Messages {
        undelivered: Vec<Message>
    }

    impl Messages {
        pub fn load() -> Messages {
            if let Ok(messages) = Messages::load_internal() {
                messages
            } else {
                Messages {
                    undelivered: Vec::new()
                }
            }
        }

        fn load_internal() -> IoResult<Messages> {
            let mut file = try!(File::open(&Path::new("data/messages.json")));
            let data = try!(file.read_to_string());
            decode(data[]).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Decoder error",
                detail: Some(e.to_string()),
            })
        }

        pub fn save(&self) -> IoResult<()> {
            try!(mkdir_recursive(&Path::new("data/"), FilePermission::all()));
            let mut f = File::create(&Path::new("data/messages.json"));
            f.write_str(encode(self)[])
        }

        pub fn add_message(&mut self, target: &str, message: &str, sender: &str) {
            self.undelivered.push(Message::new(target, message, sender))
        }

        pub fn get_messages(&mut self, user: &str) -> Vec<Message> {
            let (ret, remain) = self.undelivered.partitioned(|m| m.is_target(user));
            self.undelivered = remain;
            let _ = self.save();
            ret
        }
    }

    #[deriving(Clone, Decodable, Encodable)]
    struct Message {
        target: String,
        sender: String,
        message: String,
        time: Timespec,
    }

    impl Message {
        pub fn new(target: &str, message: &str, sender: &str) -> Message {
            Message {
                target: target.into_string(),
                sender: sender.into_string(),
                message: message.into_string(),
                time: get_time(),
            }
        }
        
        pub fn is_target(&self, user: &str) -> bool {
            self.target[] == user
        }
    }

    impl ToString for Message {
        fn to_string(&self) -> String {
            format!("{}: {}, {} said {}.", self.target, self.time, self.sender, self.message)
        }
    }
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use irc::conn::Connection;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            println!("{}", message);
            let mut args = Vec::new();
            let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
            args.push_all(msg_args[]);
            if let Some(ref suffix) = message.suffix {
                args.push(suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            super::process_internal(
                &Wrapper::new(&server), source[], message.command[], args[]
            ).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }
    
    // TODO: add tests
}