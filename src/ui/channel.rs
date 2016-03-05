extern crate gtk;

use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::signal::EntrySignals;

pub struct ChannelWidget
{
    name: String,
    pane: gtk::Box,
    backlog: gtk::TextView,
    entry: gtk::Entry,
}

impl ChannelWidget
{
    pub fn new(name: String) -> ChannelWidget
    {
        let pane = gtk::Box::new(gtk::Orientation::Vertical, 0).unwrap();
        let backlog = gtk::TextView::new().unwrap();
        let entry = gtk::Entry::new().unwrap();

        pane.pack_start(&backlog, true, true, 0);
        pane.pack_start(&entry, false, false, 0);

        ChannelWidget
        {
            name: name,
            pane: pane,
            backlog: backlog,
            entry: entry,
        }
    }

    pub fn get_pane(&self) -> &gtk::Box
    {
        &self.pane
    }

    pub fn on_entry_activate<F>(&mut self, callback: F) -> u64
        where F: Fn(gtk::Entry) + 'static
    {
        self.entry.connect_activate(callback)
    }
}
