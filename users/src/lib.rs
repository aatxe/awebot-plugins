#![feature(slicing_syntax)]
extern crate irc;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::conn::NetStream;
use irc::data::{Message, User};
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::data::user::AccessLevel::{Owner, Admin, Oper, HalfOp, Voice};
use irc::server::Server;
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

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, _: &str, command: &str, 
                                  args: &[&str]) -> IoResult<()> where T: IrcReader, U: IrcWriter {
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if msg.starts_with("@users") {
            let stringify = |users: Vec<User>| -> String {
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
                if ret[len - 2..] == "\r\n" {
                    ret.truncate(len - 4);
                } else {
                    ret.truncate(len - 2);
                }
                ret
            };
            let users = server.list_users(chan).unwrap();
            try!(server.send_privmsg(chan, format!("Users: {}", stringify(users))[]));
        }
    }
    Ok(())
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
            let message = message.unwrap();
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
