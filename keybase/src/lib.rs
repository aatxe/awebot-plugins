#![feature(if_let, slicing_syntax)]
extern crate irc;
extern crate hyper;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use hyper::Url;
use hyper::client::Request;
use hyper::header::common::connection::{Connection, ConnectionOption};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::utils::Wrapper;

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
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if tokens[0] == "@keybase" && tokens.len() == 2 {
            let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?usernames={}", 
                              tokens[1]);
            let mut req = Request::get(Url::parse(url[]).unwrap()).unwrap();
            req.headers_mut().set(Connection(vec![ConnectionOption::Close]));
            let res = req.start().unwrap().send().unwrap().read_to_string().unwrap();
            println!("{}", res);
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
    
    // TODO: add tests
}
