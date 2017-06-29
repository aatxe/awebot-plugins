extern crate irc;

use std::borrow::ToOwned;
use irc::client::prelude::*;
use irc::error;
use irc::proto::Command::{PART, PRIVMSG};

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(_, ref msg) = msg.command {
        let tokens: Vec<_> = msg.split(" ").collect();
        if server.config().is_owner(user) {
            if tokens.contains(&"join") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        server.send_join(token)?;
                    }
                }
            } else if tokens.contains(&"part") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        server.send(PART((*token).to_owned(), None))?;
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
    use irc::client::conn::MockConnection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            .. Default::default()
        }, MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
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
