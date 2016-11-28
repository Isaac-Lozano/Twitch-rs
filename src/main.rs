extern crate gtk;

pub mod ui;
pub mod twitch_chat;
pub mod twitchrs;

fn main() {
    let mut trs = twitchrs::TwitchRS::new();
    trs.run();
}
