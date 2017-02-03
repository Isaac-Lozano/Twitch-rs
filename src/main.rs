extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate twitch_chat;
extern crate twitch_api;

pub mod ui;
pub mod twitchrs;
pub mod twitch_message;
pub mod twitch_image_loader;

fn main() {
    let mut trs = twitchrs::TwitchRS::new();
    trs.run();
}
