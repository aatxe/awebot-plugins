#![feature(collections, old_io)]
extern crate irc;
extern crate rand;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::utils::Wrapper;
use rand::{thread_rng, sample};

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        if msg.starts_with("@choose ") {            
            let res = sample(&mut thread_rng(), msg[8..].split_str(" or "), 1);
            try!(server.send_privmsg(chan, &format!("{}: {}", user, res[0])));
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
