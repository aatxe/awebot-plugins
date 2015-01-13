#![allow(unstable)]
#![feature(slicing_syntax)]
extern crate irc;

use std::io::{BufferedReader, BufferedWriter, IoResult};
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
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if chan != server.config().nickname() { return Ok(()) }
        if server.config().is_owner(user) {
            if msg.starts_with("#") || msg.starts_with("$") {
                if tokens.len() > 1 {
                    try!(server.send_privmsg(if tokens[0].starts_with("$") { &tokens[0][1..] 
                                             } else { tokens[0] }, &msg[tokens[0].len()+1..]));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use irc::client::conn::Connection;
    use irc::client::data::Config;
    use irc::client::server::{IrcServer, Server};
    use irc::client::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            nickname: Some("test".to_owned()),
            ..Default::default()
        }, Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&Wrapper::new(&server), &message).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }
    
    #[test]
    fn puppet_channel() {
        let data = test_helper(":test!test@test PRIVMSG test :#test Hi there, friend.\r\n");
        assert_eq!(&data[], "PRIVMSG #test :Hi there, friend.\r\n");
    }

    #[test]
    fn puppet_query() {
        let data = test_helper(":test!test@test PRIVMSG test :$test Hi there, friend.\r\n");
        assert_eq!(&data[], "PRIVMSG test :Hi there, friend.\r\n");
    }

}
