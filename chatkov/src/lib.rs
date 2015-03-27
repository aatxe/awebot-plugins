extern crate irc;
extern crate markov;

use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Result};
use std::io::prelude::*;
use std::path::Path;
use irc::client::conn::NetStream;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use markov::Chain;

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>, 
                   message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a ServerExt<'a, T, U>, msg: &Message) -> Result<()> 
    where T: IrcRead, U: IrcWrite {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        let tokens: Vec<_> = msg.split(" ").collect();
        let path = Path::new("data/chatkov");
        if !msg.starts_with("@") {
            let mut file = try!(OpenOptions::new().write(true).append(true).open(path));
            try!(file.write_all(msg.replace(".", "\n").as_bytes()));
            try!(file.flush());
        } else if tokens[0] == "@markov" {
            let mut chain = Chain::for_strings();
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
    fn chatkov() {
        let data = test_helper(":test!test@test PRIVMSG #test :@markov\r\n"); 
        assert!(data.len() > 0)
    }
}
