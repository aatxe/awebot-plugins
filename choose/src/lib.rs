extern crate irc;
extern crate rand;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use rand::{thread_rng, sample};

#[no_mangle]
pub extern fn process(server: &IrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        if msg.starts_with("@choose ") {
            let res = sample(&mut thread_rng(), msg[8..].split(" or "), 1);
            try!(server.send_privmsg(&chan, &format!("{}: {}", user, res[0])));
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
        let server = IrcServer::from_connection(Default::default(), MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
    }

    #[test]
    fn choose_from_two() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n"]
                .contains(&&data[..]));
    }

    #[test]
    fn choose_from_three() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that or the other \
                                thing\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n",
                "PRIVMSG #test :test: the other thing\r\n"].contains(&&data[..]));
    }
}
