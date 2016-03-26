extern crate irc;
extern crate markov;

use std::fs::OpenOptions;
use std::io::Result;
use std::io::prelude::*;
use std::path::Path;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use markov::Chain;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        let tokens: Vec<_> = msg.split(" ").collect();
        let path = Path::new("data/chatkov");
        if !msg.starts_with("@") {
            let mut file = try!(OpenOptions::new().write(true).append(true).open(path));
            try!(file.write_all(msg.replace(".", "\n").as_bytes()));
            try!(file.flush());
        } else if tokens[0] == "@markov" {
            let mut chain = Chain::new();
            chain.feed_file(&path);
            let msg = if tokens.len() > 1 {
                chain.generate_str_from_token(tokens[1])
            } else {
                chain.generate_str()
            };
            try!(server.send_privmsg(&chan, &format!("{}: {}", user, if msg.len() > 0 {
                &msg[..]
            } else {
                "That seed is unknown."
            })));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use irc::client::conn::MockConnection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
    }

    #[test]
    fn chatkov() {
        let data = test_helper(":test!test@test PRIVMSG #test :@markov\r\n");
        assert!(data.len() > 0)
    }
}
