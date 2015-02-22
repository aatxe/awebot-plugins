#![feature(old_io)]
extern crate irc;
extern crate rand;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::{Command, Message};
use irc::client::data::Command::PRIVMSG;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::utils::Wrapper;
use rand::{thread_rng, Rng};

static MESSAGES: &'static [&'static str] = 
&[ "Hey, baby. Want some fuck?"
 , "If I were you, I'd have sex with me."
 , "You've got 206 bones in your body. Want one more?"
 , "Your daddy must have been a baker because you've got a nice set of buns."
 , "If I told you you had a beautiful chest, would you hold it against me?"
 , "Want to play army? I'll lay down, and you can blow the hell out of me."
 , "That shirt is very becoming on you.  If I were on you, I'd be coming too."
 , "You might not be the best looking girl here, but beauty is only a light switch away."
 , "Nice shoes. Want to screw?"
 , "I'd like to kiss you passionately on the lips. Then, I'll move up to your belly button."
 , "Something tells me you're sweet. Can I have a taste?"
 , "Do you work for UPS? Because I swear I saw you checking out my package."
 , "Do you want to come over to my place and feed your beaver some wood?"
 , "Want to play Kite? I lay down, you blow, and we'll see how high you can take me."
 , "That outfit would look great in a crumpled heap on my bedroom floor tomorrow morning."
 ];

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(_, _)) = Command::from_message(msg) {
        let mut rng = thread_rng();
        if rng.gen_weighted_bool(1000) {
            try!(server.send_privmsg(user, *rng.choose(MESSAGES).unwrap()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;
    use std::default::Default;
    use std::old_io::{MemReader, MemWriter};
    use irc::client::conn::Connection;
    use irc::client::server::{IrcServer, Server};
    use irc::client::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&Wrapper::new(&server), &message).unwrap();
        }
        let vec = server.conn().writer().get_ref().to_vec();
        String::from_utf8(vec).unwrap()
    }

    #[test]
    fn lewd() {
        let data = test_helper(":test!test@test PRIVMSG #test :test\r\n");
        assert!(super::MESSAGES.iter().map(|s| format!("PRIVMSG test :{}\r\n", s))
                .collect::<Vec<_>>().contains(&data.to_owned()) || &data[..] == "");
    }
}
