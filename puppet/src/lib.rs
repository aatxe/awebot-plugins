extern crate irc;

use irc::client::prelude::*;
use irc::error;
use irc::proto::Command::PRIVMSG;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        let tokens: Vec<_> = msg.split(" ").collect();
        if chan != server.config().nickname() { return Ok(()) }
        if server.config().is_owner(user) {
            if msg.starts_with("#") || msg.starts_with("$") {
                if tokens.len() > 1 {
                    server.send_privmsg(if tokens[0].starts_with("$") {
                        &tokens[0][1..]
                    } else {
                        tokens[0]
                    }, &msg[tokens[0].len()+1..])?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use irc::client::conn::MockConnection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            nickname: Some("test".to_owned()),
            ..Default::default()
        }, MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
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
