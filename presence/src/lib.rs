extern crate irc;

use std::borrow::ToOwned;
use std::io::{BufReader, BufWriter, Result};
use irc::client::conn::NetStream;
use irc::client::data::Command::{PART, PRIVMSG};
use irc::client::prelude::*;

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>, 
                   message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a ServerExt<'a, T, U>, msg: &Message) -> Result<()> 
    where T: IrcRead, U: IrcWrite {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(_, msg)) = Command::from_message(msg) {
        let tokens: Vec<_> = msg.split(" ").collect();
        if server.config().is_owner(user) {
            if tokens.contains(&"join") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send_join(token));
                    }
                }
            } else if tokens.contains(&"part") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send(PART((*token).to_owned(), None)));
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
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
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
    fn join() {
        let data = test_helper(":test!test@test PRIVMSG #test :join #test #test2\r\n");
        assert_eq!(&data[..], "JOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn part() {
        let data = test_helper(":test!test@test PRIVMSG #test :part #test #test2\r\n");
        assert_eq!(&data[..], "PART #test\r\nPART #test2\r\n");
    }
}
