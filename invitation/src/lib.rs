extern crate irc;

use irc::client::prelude::*;
use irc::error;
use irc::proto::Command::INVITE;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    if let INVITE(ref nick, ref chan) = msg.command {
        if nick == server.config().nickname() {
            server.send_join(chan)?;
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
            nickname: Some("test".to_owned()),
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
    fn joins_on_invite() {
        let data = test_helper(":test!test@test INVITE test #test\r\n");
        assert_eq!(&data[..], "JOIN #test\r\n");
    }

    #[test]
    fn ignores_invites_to_others() {
        let data = test_helper(":test!test@test INVITE not-test #test\r\n");
        assert!(data.is_empty());
    }

}
