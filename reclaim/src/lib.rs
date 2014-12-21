#![feature(slicing_syntax)]
extern crate irc;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use irc::conn::NetStream;
use irc::data::{Message, Response};
use irc::server::Server;
use irc::server::utils::Wrapper;

static mut count: uint =  0u;
static mut flag: bool = false;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {  
    if let Some(resp) = Response::from_message(&message) {   
        if resp == Response::ERR_NICKNAMEINUSE {
            unsafe { flag = true; }
        }
    }
    unsafe {
    if flag {
        count += 1;
        if count > 10 {
            try!(server.send_privmsg("Pidgey", format!("NS RECLAIM {} {}", 
                                                       server.config().nickname(), 
                                                       server.config().nick_password())[]));
            flag = false;
       }
    }
    }
    Ok(())
}
