#![feature(old_io)]
extern crate irc;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Message, Response};
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;

static mut count: usize =  0;
static mut flag: bool = false;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {  
    if let Some(resp) = Response::from_message(&message) {   
        if resp == Response::ERR_NICKNAMEINUSE {
            unsafe { flag = true }
        }
    }
    unsafe {
        if flag {
            count += 1;
            if count > 10 {
                try!(server.send_privmsg(
                    "Pidgey", &format!("NS RECLAIM {} {}", server.config().nickname(), 
                                       server.config().nick_password())
                ));
                flag = false;
            }
        }
    }
    Ok(())
}
