extern crate gtk;

pub mod ui;
pub mod twitch_chat;
pub mod twitchrs;

use gtk::traits::*;
use gtk::signal::Inhibit;

fn main() {
    gtk::init().expect("Could not init gtk");

    let mut trs = twitchrs::TwitchRS::new();

    let mut iters = 0;
    while gtk::main_iteration_do(true)
    {
    }
}
