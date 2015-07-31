extern crate irc;
extern crate url;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

#[no_mangle]
pub fn process<'a>(server: &'a NetIrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, S, T, U>(server: &'a S, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite, S: ServerExt<'a, T, U> + Sized {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = msg.into() {
        if msg.starts_with("@ddg ") {
            let search = utf8_percent_encode(&msg[5..], DEFAULT_ENCODE_SET);
            try!(server.send_privmsg(&chan, &format!("{}: https://duckduckgo.com/?q={}",
                                                     user, search.replace("%20", "+"))));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            Cursor::new(input.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        let vec = server.conn().writer().to_vec();
        String::from_utf8(vec).unwrap()
    }

    #[test]
    fn basic_search() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg Apple\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=Apple\r\n");
    }

    #[test]
    fn search_with_spaces() {
        let data = test_helper(":test!test@test PRIVMSG #test :@ddg !w Edward Snowden\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=!w+Edward+Snowden\r\n");
    }
}
