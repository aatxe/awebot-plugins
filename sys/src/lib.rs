#![feature(plugin)]
#![plugin(regex_macros)]
extern crate irc;
extern crate regex;

use std::borrow::ToOwned;
use std::process::Command as IoCommand;
use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> Result<()> where S: ServerExt {
    println!("!!! WARNING: You have a very dangerous system plugin loaded. !!!");
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
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
