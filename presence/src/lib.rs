#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::Command::PART;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::Server;
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
    if let ("PRIVMSG", [_, msg]) = (command, args) {
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if server.config().is_owner(user) {
            if tokens.contains(&"join") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send_join(token[]));
                    }
                }
            } else if tokens.contains(&"part") {
                for token in tokens.iter() {
                    if token.starts_with("#") {
                        try!(server.send(PART(token[], None)));
                    }
                }
            }
        }
    }
    Ok(())
}