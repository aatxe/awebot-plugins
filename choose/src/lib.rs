extern crate irc;
extern crate rand;

use irc::proto::Command::PRIVMSG;
use irc::client::prelude::*;
use irc::error;
use rand::{thread_rng, sample};

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        if msg.starts_with("@choose ") {
            let res = sample(&mut thread_rng(), msg[8..].split(" or "), 1);
            try!(server.send_privmsg(&chan, &format!("{}: {}", user, res[0])));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::thread;
    use std::time::Duration;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let config = Config {
            use_mock_connection: Some(true),
            mock_initial_value: Some(input.to_owned()),
            ..Default::default()
        };
        let server = IrcServer::from_config(config).unwrap();
        server.for_each_incoming(|message| {
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }).unwrap();
        thread::sleep(Duration::from_millis(100));
        server.log_view().sent().unwrap().iter().fold(String::new(), |mut acc, msg| {
            acc.push_str(&msg.to_string());
            acc
        })
    }

    #[test]
    fn choose_from_two() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n"]
                .contains(&&data[..]));
    }

    #[test]
    fn choose_from_three() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that or the other \
                                thing\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n",
                "PRIVMSG #test :test: the other thing\r\n"].contains(&&data[..]));
    }
}
