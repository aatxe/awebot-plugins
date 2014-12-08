#![feature(slicing_syntax)]
extern crate irc;
extern crate url;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::utils::Wrapper;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    let mut args = Vec::new();
    let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
    args.push_all(msg_args[]);
    if let Some(ref suffix) = message.suffix {
        args.push(suffix[])
    }
    let source = message.prefix.unwrap_or(String::new());
    process_internal(server, source[], message.command[], args[])
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, source: &str, command: &str,
                               args: &[&str]) -> IoResult<()> where T: IrcReader, U: IrcWriter {
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if msg.starts_with("@ddg ") {
            let search = utf8_percent_encode(msg[5..], DEFAULT_ENCODE_SET);
            try!(server.send_privmsg(chan, format!("{}: https://duckduckgo.com/?q={}", 
                                     user, std::str::replace(search[], "%20", "+")[])[]));
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
            println!("{}", message);
            let mut args = Vec::new();
            let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
            args.push_all(msg_args[]);
            if let Some(ref suffix) = message.suffix {
                args.push(suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            super::process_internal(
                &Wrapper::new(&server), source[], message.command[], args[]
            ).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }

    #[test]
    fn basic_search() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg Apple\r\n");
        assert_eq!(data[], "PRIVMSG #test :test: https://duckduckgo.com/?q=Apple\r\n"); 
    }

    #[test]
    fn search_with_spaces() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg !w Edward Snowden\r\n");
        assert_eq!(data[], "PRIVMSG #test :test: https://duckduckgo.com/?q=!w+Edward+Snowden\r\n"); 
    }
}
