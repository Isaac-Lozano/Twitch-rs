use super::error::{ClientError, ClientResult};

use std::str;

use std::collections::HashMap;
use std::cmp;

use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub struct Message
{
    pub tags: Option<HashMap<String, String>>,
    pub from: Option<String>,
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct Client
{
    nick: String,
    oauth: String,
    sock: Option<TcpStream>,
    buf: Vec<u8>,
}

impl Client
{
    pub fn new() -> Client
    {
        Client
        {
            nick: String::from("justinfan457512"),
            oauth: String::from("blah"),
            sock: None,
            buf: Vec::new(),
        }
    }

    pub fn set_login(&mut self, nick: String, oauth: String)
    {
        self.nick = nick;
        self.oauth = oauth;
    }

    pub fn connect<A: ToSocketAddrs>(&mut self, addr: A) -> ClientResult<()>
    {
        self.sock = Some(try!(TcpStream::connect(addr)));
        try!(self.handshake());

        Ok(())
    }

    pub fn send_raw(&self, s: &str) -> ClientResult<()>
    {
        if self.sock.is_none()
        {
            return Err(ClientError::InvalidStateError("Socket not connected"));
        }

        let mut sock = self.sock.as_ref().unwrap();
        try!(sock.write(format!("{}\r\n", s).as_bytes()));
        Ok(())
    }

    fn handshake(&mut self) -> ClientResult<()>
    {
        {
            try!(self.send_raw("CAP REQ :twitch.tv/commands twitch.tv/tags"));
        }

        let pass_str = format!("PASS {}", self.oauth);
        try!(self.send_raw(&pass_str));
        let nick_str = format!("NICK {}", self.nick);
        try!(self.send_raw(&nick_str));
        Ok(())
    }

    /* TODO: Error checking */
    pub fn next_message(&mut self) -> ClientResult<Message>
    {
        if self.sock.is_none()
        {
            return Err(ClientError::InvalidStateError("Socket not connected"));
        }

        let mut sock = self.sock.take().unwrap();
        let end_idx;

        /* read data until a line is read */
        loop
        {
            match get_line_end_idx(&self.buf)
            {
                Some(idx) =>
                {
                    end_idx = idx;
                    break;
                }
                None => {}
            }

            let read_buf = &mut [0u8; 512];
            let len = try!(sock.read(read_buf));
            self.buf.extend(Vec::from(&read_buf[..len]));
        }

        self.sock = Some(sock);

        /* take out the line from the buf */
        let mut line = try!(String::from_utf8(Vec::from(&self.buf[..end_idx])));
        /* Guarenteed to work because we always have a "\r\n" at the end of our line */
        self.buf = Vec::from(&self.buf[end_idx+2..]);

        /* parse message */
        let mut tags = None;
        let mut from = None;
        let cmd;
        let mut args = Vec::new();

        let mut new_line;

        /* parse tags */
        if line.starts_with("@")
        {
            let mut tag_map = HashMap::new();

            {
                let split: Vec<&str> = line[1..].splitn(2, ' ').collect();

                /* XXX */
                let tag_str = split.get(0).unwrap();
                new_line = split.get(1).unwrap().to_string();

                for tag in tag_str.split(';')
                {
                    let mut tag_split = tag.split('=');

                    let key = tag_split.next().unwrap();
                    let value = tag_split.next().unwrap();

                    tag_map.insert(key.to_string(), value.to_string());
                }
            }

            tags = Some(tag_map);

            line = new_line;
        }

        /* parse from */
        if line.starts_with(":")
        {
            {
                let mut from_split = line[1..].splitn(2, ' ');

                /* XXX */
                from = Some(from_split.next().unwrap().to_string());
                new_line = from_split.next().unwrap().to_string();
            }

            line = new_line;
        }

        /* parse command */
        {
            let mut cmd_split = line.splitn(2, ' ');
            cmd = cmd_split.next().unwrap().to_string();

            new_line = cmd_split.next().unwrap_or("").to_string();
        }
        line = new_line;

        /* parse args */
        {
            let mut arg_split = line.split(' ');
            loop
            {
                match arg_split.next()
                {
                    Some(val) =>
                    {
                        if val.starts_with(':')
                        {
                            let rest = arg_split.fold(String::from(""), |acc, s| acc + " " + s);
                            args.push(val[1..].to_string() + &rest);
                            break;
                        }
                        else
                        {
                            args.push(val.to_string());
                        }
                    }
                    None =>
                        break,
                }
            }
        }

        Ok(
            Message
            {
                tags: tags,
                from: from,
                cmd: cmd,
                args: args,
            }
        )
    }
}

fn get_line_end_idx(buf: &Vec<u8>) -> Option<usize>
{
    for idx in 0..cmp::max(buf.len(), 1) - 1
    {
        if *buf.get(idx).unwrap() == '\r' as u8 && *buf.get(idx + 1).unwrap() == '\n' as u8
        {
            return Some(idx);
        }
    }

    return None;
}
