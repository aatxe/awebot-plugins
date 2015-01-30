#![feature(collections, core, io, path, slicing_syntax)]
extern crate irc;
extern crate markov;

use std::old_io::{BufferedReader, BufferedWriter, FileAccess, FileMode, IoResult};
use std::old_io::fs::File;
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::utils::Wrapper;
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
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }

    #[test]
    fn chatkov() {
        let data = test_helper(":test!test@test PRIVMSG #test :@markov\r\n"); 
        assert!(data.len() > 0)
    }
}
