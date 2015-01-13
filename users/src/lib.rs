#![allow(unstable)]
#![feature(slicing_syntax)]
extern crate irc;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message, User};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::data::AccessLevel::*;
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        if msg.starts_with("@users") {
            let stringify = |:users: Vec<User>| -> String {
                let mut ret = String::new();
                for user in users.into_iter() {
                    if user.get_name().len() == 0 { continue }
                    for level in user.access_levels().iter() {
                        ret.push_str(match level {
                            &Owner  => "~",
                            &Admin  => "&",
                            &Oper   => "@",
                            &HalfOp => "%",
                            &Voice  => "+",
                            _      => "",
                        });
                    }
                    ret.push_str(user.get_name());
                    ret.push_str(", ");
                    if ret.len() > 300 {
                        ret.push_str("\r\n");
                    }
                }
                let len = ret.len();
                if &ret[len - 2..] == "\r\n" {
                    ret.truncate(len - 4);
                } else {
                    ret.truncate(len - 2);
                }
                ret
            };
            let users = server.list_users(chan).unwrap();
            try!(server.send_privmsg(chan, &format!("Users: {}", stringify(users))[]));
        }   
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
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
