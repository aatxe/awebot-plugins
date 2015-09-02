extern crate irc;
extern crate rand;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;
use rand::{thread_rng, Rng};

#[no_mangle]
pub extern fn process(server: &NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    // TODO: plugin functionality
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(target, msg)) = msg.into() {
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
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
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

    // TODO: add tests
}
