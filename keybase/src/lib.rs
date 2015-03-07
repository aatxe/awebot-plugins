#![feature(core, io)]
extern crate irc;
extern crate hyper;
extern crate "rustc-serialize" as rustc_serialize;

use std::io::{BufReader, BufWriter, Result};
use std::io::prelude::*;
use hyper::Url;
use hyper::client::Client;
use irc::client::conn::NetStream;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;

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
        if tokens[0] == "@keybase" && (tokens.len() == 2 || tokens.len() == 3) 
        && tokens[1].len() > 0 {
            let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?usernames={}&fields={\
                              }", tokens[1], "proofs_summary,public_keys");
            let mut client = Client::new();
            let mut result = client.get(Url::parse(&url).unwrap()).send().unwrap();
            let mut res = String::new();
            try!(result.read_to_string(&mut res));
            let lookup = data::LookUp::decode(&res);
            if let Ok(lookup) = lookup {
                if tokens.len() == 2 {
                    try!(server.send_privmsg(&chan, &format!("{}: Keybase: {} {}", user, tokens[1],
                                                             lookup.display())));
                } else if tokens[2].len() > 0 {
                    let value = lookup.display_type(tokens[2]);
                    println!("{}", tokens[2]);
                    try!(server.send_privmsg(&chan, &match value {
                        Some(ref res) if tokens[2] == "dns" || tokens[2] == "generic_web_site" => {
                            format!("{}: {} has the following domains: {}", user, tokens[1], res)
                        },
                        Some(ref res) if tokens[2] == "key" => {
                            format!("{}: {}'s fingerprint is {}.", user, tokens[1], res)
                        },
                        Some(res) => {
                            format!("{}: {} is {} on {}.", user, tokens[1], res, tokens[2])
                        },
                        None => format!("{}: {} has no proof for {}.", user, tokens[1], tokens[2]),
                    }[..]));
                }
            } else {
                try!(server.send_privmsg(&chan, &format!("{}: Something went wrong!", user)));
            }
        }  
    }
    Ok(())
}

mod data {
    use std::borrow::ToOwned;
    use std::error::Error as StdError;
    use std::io::{Error, ErrorKind, Result};
    use rustc_serialize::json::decode;

    #[derive(RustcDecodable, Debug)]
    pub struct LookUp {
        them: Option<Vec<Keybase>>
    }

    impl LookUp {
        pub fn decode(string: &str) -> Result<LookUp> {
            decode(string).map_err(|e| 
                Error::new(ErrorKind::InvalidInput, "Failed to decode keybase results.",
                           Some(e.description().to_owned()))
            )
        }

        pub fn display(&self) -> String {
            self.them.as_ref().map(|v| v[0].display()).unwrap()
        }

        pub fn display_type(&self, kind: &str) -> Option<String> {
            self.them.as_ref().map(|v| v[0].display_type(kind)).unwrap()
        }
    }

    #[derive(RustcDecodable, Debug)]
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

    #[derive(RustcDecodable, Debug)]
    pub struct PublicKeys {
        primary: PublicKey
    }

    impl PublicKeys {
        pub fn display(&self) -> String {
            self.primary.display()
        }
    }

    #[derive(RustcDecodable, Debug)]
    pub struct PublicKey {
        key_fingerprint: String,
    }

    impl PublicKey {
        pub fn display(&self) -> String {
            let len = self.key_fingerprint.len() - 16;
            self.key_fingerprint[len..].to_owned()
        }
    }

    #[derive(RustcDecodable, Debug)]
    pub struct ProofSummary {
        all: Vec<Proof>
    }

    impl ProofSummary {
        pub fn display(&self) -> String {
            let mut ret = String::new();
            for proof in self.all.iter() {
                ret.push_str(&proof.display());
                ret.push_str(" ");
            }
            if self.all.len() == 0 { return String::new() }
            let len = ret.len() - 1;
            ret.truncate(len);
            ret
        }

        pub fn display_type(&self, kind: &str) -> Option<String> {
            let mut ret = String::new();
            for proof in self.all.iter().filter(|p| &p.proof_type[..] == kind) {
                ret.push_str(&proof.nametag);
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

    #[derive(RustcDecodable, Debug)]
    pub struct Proof {
        proof_type: String,
        nametag: String,
    }

    impl Proof {
        pub fn display(&self) -> String {
            format!("{}: {}", match &self.proof_type[..] {
                "twitter"          => "Twitter",
                "github"           => "GitHub",
                "reddit"           => "Reddit",
                "hackernews"       => "HackerNews",
                "coinbase"         => "Coinbase",
                "dns"              => "Website",
                "generic_web_site" => "Website",
                _                  => &self.proof_type[..]
            }, self.nametag)
        }
    }
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
    fn keybase_lookup() {
        let data = test_helper(":test!test@test PRIVMSG #test :@keybase awe\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: Keybase: awe Twitter: aatxe GitHub: aatxe \
                            Reddit: aaronweiss74 Coinbase: coinbase/awe Website: deviant-core.net \
                            Website: pdgn.co Website: aaronweiss.us\r\n"); 
    }

    #[test]
    fn keybase_lookup_key() {
        let data = test_helper(":test!test@test PRIVMSG #test :@keybase awe key\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: awe's fingerprint is a943ba9f204c61be.\r\n");
    }

    #[test]
    fn keybase_lookup_dns() {
        let data = test_helper(":test!test@test PRIVMSG #test :@keybase awe dns\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: awe has the following domains: deviant-core.net \
                            pdgn.co aaronweiss.us\r\n");
    }

    #[test]
    fn keybase_lookup_github() {
        let data = test_helper(":test!test@test PRIVMSG #test :@keybase awe github\r\n");
        assert_eq!(&data[..], "PRIVMSG #test :test: awe is aatxe on github.\r\n");
    }
}
