#![feature(plugin)]
#![plugin(regex_macros)]
extern crate irc;
extern crate regex;

use std::borrow::ToOwned;
use std::process::Command as IoCommand;
use std::io::{BufReader, BufWriter, Result};
use irc::client::conn::NetStream;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>,
                   message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a ServerExt<'a, T, U>, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite {
    println!("!!! WARNING: You have a very dangerous system plugin loaded. !!!");
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
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
