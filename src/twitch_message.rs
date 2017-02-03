use twitch_chat::message::Message;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::str::FromStr;
use std::u8;
use std::u64;
use std::usize;

const CHAT_COLORS: [UserColor; 15] = [
    UserColor(0xFF, 0x00, 0x00),
    UserColor(0x00, 0x00, 0xFF),
    UserColor(0x00, 0x80, 0x00),
    UserColor(0xB2, 0x22, 0x22),
    UserColor(0xFF, 0x7F, 0x50),
    UserColor(0x9A, 0xCD, 0x32),
    UserColor(0xFF, 0x45, 0x00),
    UserColor(0x23, 0x8B, 0x57),
    UserColor(0xDA, 0xA5, 0x20),
    UserColor(0xD2, 0x69, 0x1E),
    UserColor(0x5F, 0x9E, 0xA0),
    UserColor(0x1E, 0x90, 0xFF),
    UserColor(0xFF, 0x69, 0xB4),
    UserColor(0x8A, 0x2B, 0xE2),
    UserColor(0x00, 0xFF, 0x7F),
];

#[derive(Clone,Debug)]
pub struct TwitchPrivmsg
{
    pub name: String,
    pub emotes: Vec<TwitchEmoteRange>,
    pub badges: Vec<TwitchBadge>,
    pub color: UserColor,
    pub to: String,
    pub message: String,
}

#[derive(Clone,Debug)]
pub struct TwitchUserState
{
    pub badges: Vec<TwitchBadge>,
    pub color: UserColor,
    pub display_name: String,
    pub emote_sets: Vec<u64>,
    pub user_id: u64,
    pub user_type: (),
}

#[derive(Clone,Debug)]
pub enum TwitchMessage
{
    TwitchPrivmsg(TwitchPrivmsg),
    TwitchEcho(TwitchPrivmsg, Vec<u64>),
    TwitchGlobalUserState(TwitchUserState),
    TwitchUserState(String, TwitchUserState),
    Unknown(String),
}

impl From<Message> for TwitchMessage
{
    fn from(msg: Message) -> Self
    {
        match msg.command.as_str()
        {
            "PRIVMSG" =>
            {
                let from = msg.from.unwrap_or(String::new());
                let badges = msg.tags.get("badges")
                                     .map(|s| s.as_str())
                                     .unwrap_or("")
                                     .split(",")
                                     .map(TwitchBadge::from_str)
                                     .filter(|r| r.is_ok())
                                     .map(|r| r.unwrap())
                                     .collect();
                let emotes = msg.tags.get("emotes")
                                     .map(|s| s.as_str())
                                     .unwrap_or("")
                                     .split("/")
                                     .map(TwitchEmoteRange::from_str)
                                     .filter(|r| r.is_ok())
                                     .map(|r| r.unwrap())
                                     .collect();
                let name = msg.tags.get("display-name")
                                   .map(|s| s.clone())
                                   .map(|s| if s.is_empty()
                                        {
                                            get_name_from_prefix(&from)
                                        }
                                        else
                                        {
                                            s
                                        })
                                   .unwrap_or("".into());
                let color = msg.tags.get("color")
                                    .map(|s| s.as_str())
                                    .map(UserColor::from_str)
                                    .and_then(|r| r.ok())
                                    .unwrap_or_else(||
                                        UserColor::from_name(&name));
                TwitchMessage::TwitchPrivmsg(
                    TwitchPrivmsg {
                        name: name,
                        emotes: emotes,
                        badges: badges,
                        color: color,
                        to: msg.args.get(0)
                                    .map(|s| s.clone())
                                    .unwrap_or(String::new()),
                        message: msg.args.get(1)
                                         .map(|s| s.clone())
                                         .unwrap_or(String::new()),
                    }
                )
            },
            "GLOBALUSERSTATE" =>
            {
                let from = msg.from.unwrap_or(String::new());
                let badges = msg.tags.get("badges")
                                     .map(|s| s.as_str())
                                     .unwrap_or("")
                                     .split(",")
                                     .map(TwitchBadge::from_str)
                                     .filter(|r| r.is_ok())
                                     .map(|r| r.unwrap())
                                     .collect();
                let emote_sets = msg.tags.get("emote-sets")
                                         .map(|s| s.as_str())
                                         .unwrap_or("")
                                         .split(",")
                                         .map(u64::from_str)
                                         .filter(|r| r.is_ok())
                                         .map(|r| r.unwrap())
                                         .collect();
                let name = msg.tags.get("display-name")
                                   .map(|s| s.clone())
                                   .map(|s| if s.is_empty()
                                        {
                                            get_name_from_prefix(&from)
                                        }
                                        else
                                        {
                                            s
                                        })
                                   .unwrap_or("".into());
                let color = msg.tags.get("color")
                                    .map(|s| s.as_str())
                                    .map(UserColor::from_str)
                                    .and_then(|r| r.ok())
                                    .unwrap_or_else(||
                                        UserColor::from_name(&name));
                let user_id = msg.tags.get("user-id")
                                      .map(String::as_str)
                                      .map(u64::from_str)
                                      .unwrap_or(Ok(0))
                                      .unwrap_or(0);
                TwitchMessage::TwitchGlobalUserState(
                    TwitchUserState {
                        badges: badges,
                        color: color,
                        display_name: name,
                        emote_sets: emote_sets,
                        user_id: user_id,
                        user_type: (),
                    }
                )
            },
            "USERSTATE" =>
            {
                let from = msg.from.unwrap_or(String::new());
                let badges = msg.tags.get("badges")
                                     .map(|s| s.as_str())
                                     .unwrap_or("")
                                     .split(",")
                                     .map(TwitchBadge::from_str)
                                     .filter(|r| r.is_ok())
                                     .map(|r| r.unwrap())
                                     .collect();
                let emote_sets = msg.tags.get("emote-sets")
                                         .map(|s| s.as_str())
                                         .unwrap_or("")
                                         .split(",")
                                         .map(u64::from_str)
                                         .filter(|r| r.is_ok())
                                         .map(|r| r.unwrap())
                                         .collect();
                let name = msg.tags.get("display-name")
                                   .map(|s| s.clone())
                                   .map(|s| if s.is_empty()
                                        {
                                            get_name_from_prefix(&from)
                                        }
                                        else
                                        {
                                            s
                                        })
                                   .unwrap_or("".into());
                let color = msg.tags.get("color")
                                    .map(|s| s.as_str())
                                    .map(UserColor::from_str)
                                    .and_then(|r| r.ok())
                                    .unwrap_or_else(||
                                        UserColor::from_name(&name));
                let user_id = msg.tags.get("user-id")
                                      .map(String::as_str)
                                      .map(u64::from_str)
                                      .unwrap_or(Ok(0))
                                      .unwrap_or(0);
                let chan = msg.args.get(0)
                                   .map(|s| s.clone())
                                   .unwrap_or(String::new());
                TwitchMessage::TwitchUserState(
                    chan,
                    TwitchUserState {
                        badges: badges,
                        color: color,
                        display_name: name,
                        emote_sets: emote_sets,
                        user_id: user_id,
                        user_type: (),
                    }
                )
            },
            _ => TwitchMessage::Unknown(msg.raw)
        }
    }
}

#[derive(Copy,Clone,Debug)]
pub struct UserColor(pub u8, pub u8, pub u8);

impl UserColor
{
    fn from_name(name: &str) -> UserColor
    {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        CHAT_COLORS[hash as usize % 15]
    }
}

impl FromStr for UserColor
{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()>
    {
        if s.len() == 7
        {
            let r = try!(u8::from_str_radix(&s[1..3], 16).or(Err(())));
            let g = try!(u8::from_str_radix(&s[3..5], 16).or(Err(())));
            let b = try!(u8::from_str_radix(&s[5..7], 16).or(Err(())));
            Ok(UserColor(r, g, b))
        }
        else
        {
            Err(())
        }
    }
}

fn get_name_from_prefix(prefix: &str) -> String
{
    let nick = prefix.split("!")
                     .next()
                     .unwrap_or("");
    let mut chars = nick.chars();
    match chars.next()
    {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct TwitchBadge
{
    pub set: String,
    pub version: String,
}

impl FromStr for TwitchBadge
{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()>
    {
        let mut split = s.splitn(2, "/");
        let set = try!(split.next().ok_or(()));
        let version = try!(split.next().ok_or(()));
        Ok (
            TwitchBadge {
                set: set.into(),
                version: version.into(),
            }
        )
    }
}

#[derive(Clone,Debug)]
pub struct TwitchEmoteRange
{
    pub id: u64,
    pub ranges: Vec<(usize, usize)>,
}

impl FromStr for TwitchEmoteRange
{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()>
    {
        let mut split = s.splitn(2, ":");
        let emote_id_str = try!(split.next().ok_or(()));
        let emote_id = try!(u64::from_str(emote_id_str).or(Err(())));

        let ranges_str = try!(split.next().ok_or(()));
        let ranges_split = ranges_str.split(",");
        let mut ranges = Vec::new();
        for range in ranges_split
        {
            let mut range_split = range.splitn(2, "-")
                                       .map(usize::from_str)
                                       .filter(|r| r.is_ok())
                                       .map(|r| r.unwrap());
            let start = try!(range_split.next().ok_or(()));
            let end = try!(range_split.next().ok_or(()));
            ranges.push((start, end));
        }

        Ok(
            TwitchEmoteRange {
                id: emote_id,
                ranges: ranges,
            }
        )
    }
}

#[derive(Clone,Debug)]
pub struct TwitchEmote
{
    pub id: u64,
    pub code: String,
}

#[test]
fn test_get_name_from_prefix()
{
    assert_eq!(get_name_from_prefix("name!name@name.tmi.twitch.tv"), String::from("Name"));
}
