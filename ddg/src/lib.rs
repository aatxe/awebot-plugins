#![feature(core, slicing_syntax)]
extern crate irc;
extern crate url;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::utils::Wrapper;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    let user = msg.get_source_nickname().unwrap_or(""); 
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        if msg.starts_with("@ddg ") {
            let search = utf8_percent_encode(&msg[5..], DEFAULT_ENCODE_SET);
            try!(server.send_privmsg(chan, &format!("{}: https://duckduckgo.com/?q={}", 
                                     user, search.replace("%20", "+"))[]));
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
    fn basic_search() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg Apple\r\n");
        assert_eq!(&data[], "PRIVMSG #test :test: https://duckduckgo.com/?q=Apple\r\n"); 
    }

    #[test]
    fn search_with_spaces() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg !w Edward Snowden\r\n");
        assert_eq!(&data[], "PRIVMSG #test :test: https://duckduckgo.com/?q=!w+Edward+Snowden\r\n"); 
    }
}
