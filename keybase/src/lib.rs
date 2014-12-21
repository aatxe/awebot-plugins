#![feature(slicing_syntax)]
extern crate irc;
extern crate hyper;
extern crate serialize;

use std::io::{BufferedReader, BufferedWriter, IoResult};
use hyper::Url;
use hyper::client::Client;
use irc::conn::NetStream;
use irc::data::Message;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::utils::Wrapper;

#[no_mangle]
pub fn process<'a>(server: &'a Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>, 
                   message: Message) -> IoResult<()> {
    let mut args = Vec::new();
    let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
    args.push_all(msg_args[]);
    if let Some(ref suffix) = message.suffix {
        args.push(suffix[])
    }
    let source = message.prefix.unwrap_or(String::new());
    process_internal(server, source[], message.command[], args[])
}

pub fn process_internal<'a, T, U>(server: &'a Wrapper<'a, T, U>, source: &str, command: &str,
                               args: &[&str]) -> IoResult<()> where T: IrcReader, U: IrcWriter {
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        let tokens: Vec<_> = msg.split_str(" ").collect();
        if tokens[0] == "@keybase" && tokens.len() == 2 {
            let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?usernames={}&fields={}", 
                              tokens[1], "proofs_summary");
            let mut client = Client::new();
            let res = client.get(Url::parse(url[]).unwrap()).send().unwrap().read_to_string().unwrap();
            let lookup = data::LookUp::decode(res[]);
            if let Ok(lookup) = lookup {
                try!(server.send_privmsg(chan, format!("{}: Keybase: {} {}", user, tokens[1],
                                                       lookup.display())[]));
            } else {
                try!(server.send_privmsg(chan, format!("{}: Something went wrong!", user)[]));
            }
        }
    }
    Ok(())
}

mod data {
    use std::io::{IoError, IoErrorKind, IoResult};
    use serialize::json::decode;

    #[deriving(Decodable, Show)]
    pub struct LookUp {
        them: Option<Vec<Keybase>>
    }

    impl LookUp {
        pub fn decode(string: &str) -> IoResult<LookUp> {
            decode(string).map_err(|e| IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to decode configuration file.",
                detail: Some(e.to_string()),
            })
        }

        pub fn display(&self) -> String {
            self.them.as_ref().map(|v| v[0].display()).unwrap()
        }
    }

    #[deriving(Decodable, Show)]
    pub struct Keybase {
        id: String,
        proofs_summary: ProofSummary
    }

    impl Keybase {
        pub fn display(&self) -> String {
            self.proofs_summary.display()
        }
    }

    #[deriving(Decodable, Show)]
    pub struct ProofSummary {
        all: Vec<Proof>
    }

    impl ProofSummary {
        pub fn display(&self) -> String {
            let mut ret = String::new();
            for proof in self.all.iter() {
                ret.push_str(proof.display()[]);
                ret.push_str(" ");
            }
            let len = ret.len() - 1;
            ret.truncate(len);
            ret
        }
    }

    #[deriving(Decodable, Show)]
    pub struct Proof {
        proof_type: String,
        nametag: String,
    }

    impl Proof {
        pub fn display(&self) -> String {
            format!("{}: {}", match self.proof_type[] {
                "twitter"          => "Twitter",
                "github"           => "GitHub",
                "reddit"           => "Reddit",
                "hackernews"       => "HackerNews",
                "coinbase"         => "Coinbase",
                "dns"              => "Website",
                "generic_web_site" => "Website",
                _                  => self.proof_type[]
            }, self.nametag)
        }
    }
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use irc::conn::Connection;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            MemReader::new(input.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            println!("{}", message);
            let mut args = Vec::new();
            let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
            args.push_all(msg_args[]);
            if let Some(ref suffix) = message.suffix {
                args.push(suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            super::process_internal(
                &Wrapper::new(&server), source[], message.command[], args[]
            ).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }
    
    // TODO: add tests
}
