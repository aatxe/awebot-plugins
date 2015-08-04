#![feature(plugin)]
#![plugin(regex_macros)]
extern crate irc;
extern crate regex;

use std::borrow::ToOwned;
use std::process::Command as IoCommand;
use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;

#[no_mangle]
pub fn process(server: &NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    println!("!!! WARNING: You have a very dangerous system plugin loaded. !!!");
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = msg.into() {
        if server.config().is_owner(user) {
            let tokens: Vec<_> = msg.split(" ").collect();
            if tokens[0] == "%" {
                let msg = match IoCommand::new(tokens[1]).args(&tokens[2..]).output() {
                    Ok(output) => regex!(r"[\s]").replace_all(
                        &String::from_utf8_lossy(&output.stdout).to_owned(), " "
                    ).to_owned(),
                    Err(e) => format!("Failed to execute command; {}.", e),
                };
                if &msg[..] == "" {
                    try!(server.send_privmsg(&chan, "No output."));
                } else {
                    try!(server.send_privmsg(&chan, &msg));
                }
            }
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
