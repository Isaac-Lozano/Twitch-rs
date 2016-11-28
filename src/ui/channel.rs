extern crate gtk;

use gtk::prelude::*;

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
        let pane = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let backlog = gtk::TextView::new();
        let entry = gtk::Entry::new();

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

    pub fn get_entry(&self) -> &gtk::Entry
    {
        &self.entry
    }

    pub fn println(&self, line: &str)
    {
        /* Why doesn't this need to be mut? */
        let buf = self.backlog.get_buffer().unwrap();
        let mut end = buf.get_end_iter();
        buf.insert(&mut end, line);
        buf.insert(&mut end, "\n");
    }

    pub fn on_entry_activate<F>(&mut self, callback: F) -> u64
        where for <'r> F: Fn(&'r gtk::Entry) + 'static
    {
        self.entry.connect_activate(callback)
    }
}
