extern crate irc;
extern crate rand;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use rand::{thread_rng, Rng};

#[no_mangle]
pub extern fn process(server: &IrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref target, ref msg) = msg.command {
        let resp = if target.starts_with("#") {
            &target[..]
        } else {
            user
        };
        if msg.contains("snake") {
            let mut rng = thread_rng();
            let snake = {
                let mut tmp = String::new();
                tmp.push_str("METAL GE");
                for _ in 0 .. rng.gen_range(1, 4) {
                    tmp.push_str("A");
                }
                for _ in 0 .. rng.gen_range(5, 17) {
                    tmp.push_str("R");
                }
                tmp
            };
            try!(server.send_privmsg(resp, &snake));
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

    // TODO: add tests
}
