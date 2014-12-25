#![feature(slicing_syntax)]
extern crate irc;
extern crate hyper;
extern crate "rustc-serialize" as rustc_serialize;

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
        if tokens[0] == "@keybase" && (tokens.len() == 2 || tokens.len() == 3) 
        && tokens[1].len() > 0 {
            let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?usernames={}&fields={}", 
                              tokens[1], "proofs_summary,public_keys");
            let mut client = Client::new();
            let res = client.get(Url::parse(url[]).unwrap()).send().unwrap().read_to_string().unwrap();
            let lookup = data::LookUp::decode(res[]);
            if let Ok(lookup) = lookup {
                if tokens.len() == 2 {
                    try!(server.send_privmsg(chan, format!("{}: Keybase: {} {}", user, tokens[1],
                                                           lookup.display())[]));
                } else if tokens[2].len() > 0 {
                    let value = lookup.display_type(tokens[2]);
                    println!("{}", tokens[2]);
                    try!(server.send_privmsg(chan, match value {
                        Some(ref res) if tokens[2] == "dns" || tokens[2] == "generic_web_site" => {
                            format!("{}: {} has the following domains: {}", user, tokens[1], res)
                        },
                        Some(ref res) if tokens[2] == "key" => {
                            format!("{}: {}'s fingerprint is {}.", user, tokens[1], res)
                        },
                        Some(res) => format!("{}: {} is {} on {}.", user, tokens[1], res, tokens[2]),
                        None => format!("{}: {} has no proof for {}.", user, tokens[1], tokens[2]),
                    }[]));
                }
            } else {
                try!(server.send_privmsg(chan, format!("{}: Something went wrong!", user)[]));
            }
        }
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::io::{IoError, IoErrorKind, IoResult};
    use rustc_serialize::json::decode;

    #[deriving(RustcDecodable, Show)]
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

        pub fn display_type(&self, kind: &str) -> Option<String> {
            self.them.as_ref().map(|v| v[0].display_type(kind)).unwrap()
        }
    }

    #[deriving(RustcDecodable, Show)]
    pub struct Keybase {
        id: String,
        public_keys: PublicKeys,
        proofs_summary: ProofSummary,
    }

    impl Keybase {
        pub fn display(&self) -> String {
            self.proofs_summary.display()
        }

        pub fn display_type(&self, kind: &str) -> Option<String> {
            if kind == "key" {
                Some(self.public_keys.display())
            } else {
                self.proofs_summary.display_type(kind)
            }
        }
    }

    #[deriving(RustcDecodable, Show)]
    pub struct PublicKeys {
        primary: PublicKey
    }

    impl PublicKeys {
        pub fn display(&self) -> String {
            self.primary.display()
        }
    }

    #[deriving(RustcDecodable, Show)]
    pub struct PublicKey {
        key_fingerprint: String,
    }

    impl PublicKey {
        pub fn display(&self) -> String {
            let len = self.key_fingerprint.len() - 16;
            self.key_fingerprint[len..].to_owned()
        }
    }

    #[deriving(RustcDecodable, Show)]
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
            if self.all.len() == 0 { return String::new() }
            let len = ret.len() - 1;
            ret.truncate(len);
            ret
        }

        pub fn display_type(&self, kind: &str) -> Option<String> {
            let mut ret = String::new();
            for proof in self.all.iter().filter(|p| p.proof_type[] == kind) {
                ret.push_str(proof.nametag[]);
                ret.push_str(" ");
            }
            let len = ret.len() - 1;
            if len > 0 {
                ret.truncate(len);
                Some(ret)   
            } else {
                None   
            }
        }
    }

    #[deriving(RustcDecodable, Show)]
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
            let message = message.unwrap();
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
