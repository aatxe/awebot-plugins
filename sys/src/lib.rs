#![feature(collections, core, old_io, plugin)]
#![plugin(regex_macros)]
extern crate irc;
extern crate regex;

use std::borrow::ToOwned;
use std::old_io::Command as IoCommand;
use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    println!("!!! WARNING: You have a very dangerous system plugin loaded. !!!"); 
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        if server.config().is_owner(user) {
            let tokens: Vec<_> = msg.split_str(" ").collect();
            if tokens[0] == "%" {
                let msg = match IoCommand::new(tokens[1]).args(&tokens[2..]).spawn() {
                    Ok(mut p) => if let Ok(vec) = p.stdout.as_mut().unwrap().read_to_end() {
                        let re = regex!(r"[\s]");
                        re.replace_all(&String::from_utf8_lossy(&vec).to_owned(), " ")
                          .to_owned()
                    } else {
                        format!("Failed to execute command for an unknown reason.")
                    },
                    Err(e) => format!("Failed to execute command; {}.", e),
                };
                if &msg[..] == "" {
                    try!(server.send_privmsg(chan, "No output."));
                } else {
                    try!(server.send_privmsg(chan, &msg));
                }
            }
        } 
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::old_io::{MemReader, MemWriter};
    use irc::client::conn::Connection;
    use irc::client::server::{IrcServer, Server};
    use irc::client::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&Wrapper::new(&server), &message).unwrap();
        }
        let vec = server.conn().writer().get_ref().to_vec();
        String::from_utf8(vec).unwrap() 
    }
    
    // TODO: add tests
}
