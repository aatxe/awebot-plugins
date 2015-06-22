extern crate irc;

use std::io::{BufReader, BufWriter, Result};
use irc::client::conn::NetStream;
use irc::client::prelude::*;

static mut count: usize =  0;
static mut flag: bool = false;

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>,
                   message: Message) -> Result<()> {
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
