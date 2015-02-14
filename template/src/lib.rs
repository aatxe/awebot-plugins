#![feature(slicing_syntax)]
extern crate irc;

use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use irc::client::conn::NetStream;
use irc::client::data::Message;
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, msg: &Message) -> IoResult<()> 
    where T: IrcReader, U: IrcWriter {
    // TODO: plugin functionality
    Ok(())
}

#[cfg(test)]
mod test {
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
    
    // TODO: add tests
}
