extern crate irc;
extern crate chrono;
extern crate rustc_serialize;

use irc::client::prelude::*;
use irc::error;
use irc::proto::Command::PRIVMSG;

#[no_mangle]
pub extern fn process(server: &IrcServer, message: &Message) -> error::Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<S>(server: &S, msg: &Message) -> error::Result<()> where S: ServerExt {
    let user = msg.source_nickname().unwrap_or("");
    println!("{:?}", msg);
    if let PRIVMSG(ref chan, ref msg) = msg.command {
        let replyto = if chan == server.config().nickname() {
            user
        } else {
            &chan[..]
        };

        if let Some(tz) = msg.splitn(2, "@mytzis ").nth(1) {
            if let Ok(offset) = parse_tz(tz.trim()) {
                let response = if offset.abs() >= 86400 {
                    format!("{}: Chrono can't take it if you're a day or more behind UTC.", user)
                } else {
                    match data::TzOf::new(user, offset).save() {
                       Ok(_) => format!("{}: OK.", user),
                       Err(_) => format!("{}: Something bad happened.", user)
                    }
                };
                server.send_privmsg(replyto, &response)?;
            } else {
                server.send_privmsg(replyto, &format!("{}: Expecting a offset from UTC in the form +/-HHmm.", user))?;
            }
        } else if let Some(who) = msg.splitn(2, "@timefor ").nth(1) {
            let response = match data::TzOf::load(user) {
                Ok(tzof) =>
                    format!("{}: It is {} where {} is.", user, tzof.time_now().to_rfc2822(), who),
                Err(_) =>
                    format!("{}: I don't know {}'s timezone.", user, who)
            };
            server.send_privmsg(replyto, &response)?;
        }
    }
    Ok(())
}

fn parse_tz(tz: &str) -> std::result::Result<i32, ()> {
    if tz.len() != 5 {
        Err(())
    } else {
        Ok(match &tz[0..1] { "+" => Ok(1), "-" => Ok(-1), _ => Err(()) }?
           * (tz[1..3].parse::<i32>().map_err(|_| ())? * 3600
           + tz[3..5].parse::<i32>().map_err(|_| ())? * 60))
    }
}

mod data {
    use std::borrow::ToOwned;
    use std::fs::{File, create_dir_all};
    use std::io::{Error, ErrorKind, Result};
    use std::io::prelude::*;
    use std::path::Path;
    use rustc_serialize::json::{decode, encode};
    use chrono::datetime::DateTime;
    use chrono::offset::TimeZone;
    use chrono::offset::utc::UTC;
    use chrono::offset::fixed::FixedOffset;

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct TzOf {
        pub nickname: String,
        pub seconds_ahead: i32,
    }

    impl TzOf {
        pub fn new(nickname: &str, seconds_ahead: i32) -> Self {
            TzOf { nickname: nickname.to_lowercase(), seconds_ahead: seconds_ahead }
        }

        pub fn offset(&self) -> FixedOffset {
            FixedOffset::east(self.seconds_ahead)
        }

        pub fn time_from<Tz>(&self, from: &DateTime<Tz>) -> DateTime<FixedOffset>
            where Tz: TimeZone {
            from.with_timezone(&self.offset())
        }

        pub fn time_now(&self) -> DateTime<FixedOffset> {
            self.time_from(&UTC::now())
        }

        pub fn load(nickname: &str) -> Result<Self> {
            let mut path = "data/timefor/".to_owned();
            path.push_str(&nickname.to_lowercase());
            path.push_str(".json");
            let mut file = try!(File::open(Path::new(&path)));
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode timefor data."
            ))
        }

        pub fn save(&self) -> Result<()> {
            let mut path = "data/timefor/".to_owned();
            try!(create_dir_all(Path::new(&path)));
            path.push_str(&self.nickname);
            path.push_str(".json");
            let mut f = try!(File::create(&Path::new(&path)));
            try!(f.write_all(try!(encode(self).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to encode timefor data."
            ))).as_bytes()));
            f.flush()
        }
    }
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use irc::client::conn::MockConnection;
    use irc::client::prelude::*;
    use ::data;
    use chrono::offset::utc::UTC;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
            nickname: Some("bot".to_owned()),
            .. Default::default()
        }, MockConnection::new(input));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        server.conn().written(server.config().encoding()).unwrap()
    }

    #[test]
    fn test_positive() {
        assert_eq!(test_helper(":target1! PRIVMSG #a :@mytzis +1177\r\n"), "PRIVMSG #a :target1: OK.\r\n");
        assert_eq!(data::TzOf::load("target1").unwrap().seconds_ahead, 11 * 3600 + 77 * 60);
    }

    #[test]
    fn test_negative() {
        assert_eq!(test_helper(":target2! PRIVMSG #a :@mytzis -1177\r\n"), "PRIVMSG #a :target2: OK.\r\n");
        assert_eq!(data::TzOf::load("target2").unwrap().seconds_ahead, -1 * (11 * 3600 + 77 * 60));
    }

    #[test]
    fn test_time() {
        assert_eq!(test_helper(":target3! PRIVMSG #a :@mytzis -0801\r\n"), "PRIVMSG #a :target3: OK.\r\n");
        let now = UTC::now();
        assert_eq!(data::TzOf::load("target3").unwrap().time_from(&now).naive_local().timestamp() - now.timestamp(), -1 * (08 * 3600 + 1 * 60));
    }
}
