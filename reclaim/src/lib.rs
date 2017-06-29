extern crate irc;

use irc::client::prelude::*;
use irc::error;

static mut count: usize =  0;
static mut flag: bool = false;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
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
