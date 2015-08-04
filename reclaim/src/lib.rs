extern crate irc;

use std::io::Result;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;

static mut count: usize =  0;
static mut flag: bool = false;

#[no_mangle]
pub fn process(server: &NetIrcServer, message: Message) -> Result<()> {
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
