#![feature(if_let, slicing_syntax)]
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
                                     user, search[])[]));
        }
    }
    Ok(())
}
