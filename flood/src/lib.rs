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
        if server.config().is_owner(user) {
            let tokens: Vec<_> = msg.split_str(" ").collect();
            if tokens.len() >= 3 && tokens[0] == "@flood" {
                let target = tokens[1];
                if let Some(n) = tokens[2].parse() {
                    for i in range(0u8, n) {
                        if tokens.len() == 3 {
                            try!(server.send_privmsg(target, &format!("@flood ({})", i)[]));
                        } else {
                            try!(server.send_privmsg(target, 
                                            &msg[(9 + tokens[1].len() + tokens[2].len())..]));
                        }
                    }
                } else {
                    try!(server.send_privmsg(if chan == server.config().nickname() { user } 
                         else { chan }, &format!("{} is not a number.", tokens[2])[]));
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
            nickname: Some("flood".to_owned()),
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
    fn flood_default_msg() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test 2\r\n");
        assert_eq!(&data[], "PRIVMSG #test :@flood (0)\r\nPRIVMSG #test :@flood (1)\r\n");
    }

    #[test]
    fn flood_defined_msg() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test 2 this is a test\r\n");
        assert_eq!(&data[], "PRIVMSG #test :this is a test\r\nPRIVMSG #test :this is a test\r\n");
    }

    #[test]
    fn flood_not_a_number_chan() {
        let data = test_helper(":test!test@test PRIVMSG #test :@flood #test q this is a test\r\n");
        assert_eq!(&data[], "PRIVMSG #test :q is not a number.\r\n");
    }
    
    #[test]
    fn flood_not_a_number_query() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test q this is a test\r\n");
        assert_eq!(&data[], "PRIVMSG test :q is not a number.\r\n");
    }

    #[test]
    fn flood_not_owner() {
        let data = test_helper(":test2!test@test PRIVMSG flood :@flood #test 3\r\n");
        assert_eq!(&data[], "");
    }
}
