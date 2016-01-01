extern crate irc;
extern crate url;

use std::io::Result;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::client::server::NetIrcServer;
use url::Url;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

#[no_mangle]
pub extern fn process(server: &NetIrcServer, message: Message) -> Result<()> {
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
        } else if msg.contains("google.com") {
            for url in find_urls(&msg).into_iter() {
                if url.domain().is_some() && url.domain().unwrap().ends_with("google.com") {
                    let frag = url.clone().fragment.unwrap_or_default();
                    if frag.contains("q=") {
                        let item = match frag.find("q=") {
                            Some(start) => match frag.find("&") {
                                Some(end) => &frag[start+2..end],
                                None => &frag[start+2..],
                            },
                            None => ""
                        };
                        try!(server.send_privmsg(&chan,
                                 &format!("{}: https://duckduckgo.com/?q={}", user, item.replace(" ", "+"))))
                    } else if let Some(pairs) = url.query_pairs() {
                        if let Some(tup) = pairs.iter().find(|tup| &tup.0[..] == "q") {
                            try!(server.send_privmsg(&chan,
                                 &format!("{}: https://duckduckgo.com/?q={}", user, tup.1.replace(" ", "+"))))
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn tokenize(msg: &str) -> Vec<&str> {
    msg.split(' ').map(|s| s.trim_matches(
        |c| ['(', ')', '{', '}', '[', ']', '<', '>', '.', '!', '?', ','].contains(&c)
    )).collect()
}

pub fn find_urls(msg: &str) -> Vec<Url> {
    tokenize(msg).iter().map(|s| Url::parse(s)).filter(|r| r.is_ok()).map(|r| r.unwrap()).collect()
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;
    use url::Url;

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

    #[test]
    fn tokenize() {
        assert_eq!(super::tokenize("this is a test."), vec!("this", "is", "a", "test"));
        assert_eq!(super::tokenize("this is (a test)."), vec!("this", "is", "a", "test"));
        assert_eq!(super::tokenize("<<this is a [complicated] test, I suppose."),
                   vec!("this", "is", "a", "complicated", "test", "I", "suppose"));
    }

    #[test]
    fn find_urls() {
        assert_eq!(super::find_urls("this is http://test.com."), vec!(Url::parse("http://test.com").unwrap()));
        assert_eq!(super::find_urls("this is another (http://test.com/)."), vec!(Url::parse("http://test.com").unwrap()));
    }

    #[test]
    fn correct_google() {
        let data = test_helper(":test!test@test PRIVMSG #test :http://google.com/?q=test\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=test\r\n");
        let data2 = test_helper(":test!test@test PRIVMSG #test :Some text http://google.com/?q=test. Blah.\r\n");
        assert_eq!(&data2[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=test\r\n");
        let data3 = test_helper(":test!test@test PRIVMSG #test :https://www.google.com/?q=test\r\n");
        assert_eq!(&data3[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=test\r\n");
        let data4 = test_helper(":test!test@test PRIVMSG #test :https://www.google.com/?q=with+space\r\n");
        assert_eq!(&data4[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=with+space\r\n");
        let data5 = test_helper(":test!test@test PRIVMSG #test :https://www.google.com/#q=with+space\r\n");
        assert_eq!(&data5[..], "PRIVMSG #test :test: https://duckduckgo.com/?q=with+space\r\n");
    }
}
