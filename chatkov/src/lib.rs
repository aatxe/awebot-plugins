#![allow(unstable)]
#![feature(slicing_syntax)]
extern crate irc;
extern crate markov;

use std::io::{BufferedReader, BufferedWriter, FileAccess, FileMode, IoResult};
use std::io::fs::File;
use irc::conn::NetStream;
use irc::data::{Command, Message};
use irc::data::Command::PRIVMSG;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::utils::Wrapper;
use markov::Chain;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        let tokens: Vec<_> = msg.split_str(" ").collect();
        let path = Path::new("data/chatkov");
        if !msg.starts_with("@") {
            let mut file = File::open_mode(&path, FileMode::Append, FileAccess::Write);
            try!(file.write_line(&msg.replace(".", "\n")[]));
        } else if tokens[0] == "@markov" {
            let mut chain = Chain::for_strings();
            chain.feed_file(&path);
            let msg = if tokens.len() > 1 {
                chain.generate_str_from_token(tokens[1])
            } else {
                chain.generate_str()
            };
            try!(server.send_privmsg(chan, &format!("{}: {}", user, if msg.len() > 0 { 
                &msg[] 
            } else {
                "That seed is unknown."
            })[]));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use irc::conn::Connection;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&Wrapper::new(&server), &message).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }
    
    // TODO: add tests
}
