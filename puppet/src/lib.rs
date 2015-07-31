extern crate irc;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;

#[no_mangle]
pub fn process<'a>(server: &'a NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = msg.into() {
        let tokens: Vec<_> = msg.split(" ").collect();
        if chan != server.config().nickname() { return Ok(()) }
        if server.config().is_owner(user) {
            if msg.starts_with("#") || msg.starts_with("$") {
                if tokens.len() > 1 {
                    try!(server.send_privmsg(if tokens[0].starts_with("$") { &tokens[0][1..] }
                                             else { tokens[0] }, &msg[tokens[0].len()+1..]));
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
            nickname: Some("test".to_owned()),
            ..Default::default()
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
    fn puppet_channel() {
        let data = test_helper(":test!test@test PRIVMSG test :#test Hi there, friend.\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :Hi there, friend.\r\n");
    }

    #[test]
    fn puppet_query() {
        let data = test_helper(":test!test@test PRIVMSG test :$test Hi there, friend.\r\n");
        assert_eq!(&data[..], "PRIVMSG test :Hi there, friend.\r\n");
    }

}
