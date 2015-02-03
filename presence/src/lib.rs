#![feature(collections, slicing_syntax)]
extern crate irc;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::{PART, PRIVMSG};
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
    if let Ok(PRIVMSG(_, msg)) = Command::from_message(msg) {
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if server.config().is_owner(user) {
            if tokens.contains(&"join") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send_join(&token[]));
                    }
                }
            } else if tokens.contains(&"part") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send(PART(&token[], None)));
                    }
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
    use std::old_io::{MemReader, MemWriter};
    use irc::client::conn::Connection;
    use irc::client::data::Config;
    use irc::client::server::{IrcServer, Server};
    use irc::client::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            .. Default::default()
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
    fn join() {
        let data = test_helper(":test!test@test PRIVMSG #test :join #test #test2\r\n");
        assert_eq!(&data[], "JOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn part() {
        let data = test_helper(":test!test@test PRIVMSG #test :part #test #test2\r\n");
        assert_eq!(&data[], "PART #test\r\nPART #test2\r\n");
    }
}
