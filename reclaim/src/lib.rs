extern crate irc;

use std::io::Result;
use irc::client::prelude::*;

static mut count: usize =  0;
static mut flag: bool = false;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: Message) -> Result<()> {
    if let Command::Response(Response::ERR_NICKNAMEINUSE, _, _) = message.command {
        unsafe { flag = true }
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
