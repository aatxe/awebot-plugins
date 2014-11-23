#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::io::{BufferedStream, IoResult};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::IrcStream;
use irc::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedStream<NetStream>>, message: Message) 
    -> IoResult<()> {
    let mut args = Vec::new();
    let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
    args.push_all(msg_args[]);
    if let Some(ref suffix) = message.suffix {
        args.push(suffix[])
    }
    let source = message.prefix.unwrap_or(String::new());
    process_internal(server, source[], message.command[], args[])
}

pub fn process_internal<'a, T>(server: &'a Wrapper<'a, T>, source: &str, command: &str,
                               args: &[&str]) -> IoResult<()> where T: IrcStream {
    // TODO: plugin functionality
    Ok(())
}
