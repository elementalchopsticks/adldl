mod dcc;
mod nibl;

use dcc::Dcc;
pub use nibl::search;

use std::io::{Read, Write};
use std::lazy::SyncLazy;
use std::net::{Shutdown, TcpStream};
use std::rc::Rc;
use std::str::from_utf8;

use anyhow::{Context, Result};
use regex::Regex;

pub struct Bot {
    pub name: String,
}

pub struct Pack {
    pub bot: Rc<Bot>,
    pub number: usize,
    pub name: String,
    pub size: String,
}

static PING_RE: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r#"^(?:\S+ )?PING (\S+)"#).unwrap());
static MODE_RE: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r#"^(?:\S+ )?MODE"#).unwrap());
static DCC_RE: SyncLazy<Regex> = SyncLazy::new(|| {
    Regex::new(r#"^(?:\S+ )?PRIVMSG.*(?:DCC|dcc) (?:SEND|send) "?(.*?)"? (\d+) (\d+) (\d+)"#)
        .unwrap()
});

impl Pack {
    pub fn download(self) -> Result<()> {
        println!("Connecting to IRC...");
        let mut stream =
            TcpStream::connect("irc.rizon.net:6667").context("failed to connect to IRC")?;
        send_message(&mut stream, "USER AnimeFan42 0 * AnimeFan42")?;
        send_message(&mut stream, "NICK AnimeFan42")?;

        let mut remainder = Vec::new();
        let dcc = 'outer: loop {
            for message in get_messages(&mut stream, &mut remainder)? {
                if let Some(dcc) =
                    handle_message(&mut stream, &message, &self.bot.name, self.number)?
                {
                    break 'outer dcc;
                }
            }
        };

        send_message(&mut stream, "QUIT")?;
        stream.shutdown(Shutdown::Both)?;

        println!("Downloading file...");
        dcc.download()?;
        println!("Success!");

        Ok(())
    }
}

fn get_messages(stream: &mut TcpStream, remainder: &mut Vec<u8>) -> Result<Vec<String>> {
    let mut buf = [0; 1024];
    let count = stream.read(&mut buf)?;

    let mut messages = Vec::new();

    let mut it = buf[..count].split(|elem| *elem == b'\n').peekable();
    while let Some(line) = it.next() {
        if it.peek().is_none() {
            remainder.append(&mut line.to_owned());
            continue;
        }

        let message = if !remainder.is_empty() {
            let full = [remainder.trim_ascii_start(), line.trim_ascii_end()].concat();
            *remainder = Vec::new();
            String::from_utf8(full).unwrap()
        } else {
            from_utf8(line).unwrap().trim().to_owned()
        };

        messages.push(message);
    }

    Ok(messages)
}

fn handle_message(
    stream: &mut TcpStream,
    message: &str,
    bot: &str,
    packnum: usize,
) -> Result<Option<Dcc>> {
    if PING_RE.is_match(message) {
        let caps = PING_RE.captures(message).unwrap();
        send_message(stream, &format!("PONG {}", &caps[1]))?;
    } else if MODE_RE.is_match(message) {
        send_message(stream, "JOIN #nibl")?;
        send_message(stream, &format!("PRIVMSG {} :xdcc send #{}", bot, packnum))?;
        println!("Waiting for DCC connection...");
    } else if DCC_RE.is_match(message) {
        let caps = DCC_RE.captures(message).unwrap();
        let filename = &caps[1];
        let ip = &caps[2];
        let port = &caps[3];
        let size = &caps[4];

        let dcc = Dcc::new(
            filename.to_owned(),
            &format!("{}:{}", ip, port),
            size.parse().unwrap(),
        )?;

        return Ok(Some(dcc));
    }

    Ok(None)
}

fn send_message(stream: &mut TcpStream, message: &str) -> Result<()> {
    stream.write_all(message.as_bytes())?;
    stream.write_all(&[b'\r', b'\n'])?;
    Ok(())
}
