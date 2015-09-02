extern crate irc;

use std::io::Result;
use irc::client::data::User;
use irc::client::data::AccessLevel::*;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;

#[no_mangle]
pub extern fn process(server: &NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    if let Ok(PRIVMSG(chan, msg)) = msg.into() {
        if msg.starts_with("@users") {
            let stringify = |users: Vec<User>| -> String {
                let mut ret = String::new();
                for user in users.into_iter() {
                    if user.get_nickname().len() == 0 { continue }
                    for level in user.access_levels().iter() {
                        ret.push_str(match level {
                            &Owner  => "~",
                            &Admin  => "&",
                            &Oper   => "@",
                            &HalfOp => "%",
                            &Voice  => "+",
                            _      => "",
                        });
                    }
                    ret.push_str(user.get_nickname());
                    ret.push_str(", ");
                    if ret.len() > 300 {
                        ret.push_str("\r\n");
                    }
                }
                let len = ret.len();
                if &ret[len - 2..] == "\r\n" {
                    ret.truncate(len - 4);
                } else {
                    ret.truncate(len - 2);
                }
                ret
            };
            let users = server.list_users(&chan).unwrap();
            try!(server.send_privmsg(&chan, &format!("Users: {}", stringify(users))));
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
