use ui::main_window::MainWindow;
use twitch_chat::client::Client;

use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk;

pub struct TwitchRS
{
    window: Rc<RefCell<MainWindow>>,
    client: Client,
}

impl TwitchRS
{
    pub fn new() -> TwitchRS
    {
        gtk::init().expect("Could not init gtk");

        let mut trs = TwitchRS
        {
            window: Rc::new(RefCell::new(MainWindow::new())),
            client: Client::new(),
        };

        trs.setup_callbacks();

        return trs;
    }

    fn twitch_thread()
    {
    }

    fn setup_callbacks(&mut self)
    {
        let window_clone = self.window.clone();
        self.window.borrow_mut().on_status_entry(move |entry_ref|
            {
                window_clone.borrow_mut().status_log_line("TEST");
                entry_ref.set_text("");
            });
    }

    pub fn run(&mut self)
    {
        while gtk::main_iteration_do(true)
        {
        }
    }
}

