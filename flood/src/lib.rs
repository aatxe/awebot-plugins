extern crate irc;

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
        if server.config().is_owner(user) {
            let tokens: Vec<_> = msg.trim_right().split(" ").collect();
            if tokens.len() >= 3 && tokens[0] == "@flood" {
                let target = tokens[1];
                if let Ok(n) = tokens[2].parse() {
                    for i in 0..n {
                        if tokens.len() == 3 {
                            try!(server.send_privmsg(target, &format!("@flood ({})", i)));
                        } else {
                            try!(server.send_privmsg(target,
                                            &msg[(9 + tokens[1].len() + tokens[2].len())..]));
                        }
                    }
                } else {
                    try!(server.send_privmsg(if &chan[..] == server.config().nickname() { user }
                         else { &chan }, &format!("{} is not a number.", tokens[2])));
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
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            nickname: Some("flood".to_owned()),
            .. Default::default()
        }, Connection::new(
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
    #[test]
    fn flood_default_msg() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test 2\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :@flood (0)\r\nPRIVMSG #test :@flood (1)\r\n");
    }

    #[test]
    fn flood_defined_msg() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test 2 this is a test\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :this is a test\r\nPRIVMSG #test :this is a test\r\n");
    }

    #[test]
    fn flood_not_a_number_chan() {
        let data = test_helper(":test!test@test PRIVMSG #test :@flood #test q this is a test\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :q is not a number.\r\n");
    }

    #[test]
    fn flood_not_a_number_query() {
        let data = test_helper(":test!test@test PRIVMSG flood :@flood #test q this is a test\r\n");
        assert_eq!(&data[..], "PRIVMSG test :q is not a number.\r\n");
    }

    #[test]
    fn flood_not_owner() {
        let data = test_helper(":test2!test@test PRIVMSG flood :@flood #test 3\r\n");
        assert_eq!(&data[..], "");
    }
}
