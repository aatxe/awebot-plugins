#![feature(if_let, phase, slicing_syntax)]
extern crate irc;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::io::{Command, BufferedStream, IoResult};
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::IrcStream;
use irc::server::Server;
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
    println!("!!! WARNING: You have a very dangerous system plugin loaded. !!!");
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if server.config().is_owner(user) {
            let tokens: Vec<_> = msg.split_str(" ").collect();
            if tokens[0] == "%" {
                let msg = match Command::new(tokens[1]).args(tokens[2..]).spawn() {
                    Ok(mut p) => if let Ok(vec) = p.stdout.as_mut().unwrap().read_to_end() {
                        let re = regex!(r"[\s]");
                        re.replace_all(String::from_utf8_lossy(vec[]).into_string()[], " ").into_string()
                    } else {
                        format!("Failed to execute command for an unknown reason.")
                    },
                    Err(e) => format!("Failed to execute command; {}.", e),
                };
                if msg[] == "" {
                    try!(server.send_privmsg(chan, "No output."));
                } else {
                    try!(server.send_privmsg(chan, msg[]));
                }
            }
        }
    }
    Ok(())
}
