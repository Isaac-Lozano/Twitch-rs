use twitch_message::{TwitchMessage, TwitchEmoteRange};
use twitch_image_loader::TwitchImageLoader;

use gdk_pixbuf;
use gtk;
use gtk::prelude::*;
use gdk::enums::key;

use std::collections::VecDeque;

use std::rc::Rc;
use std::cell::{Cell, RefCell};

struct ChannelPanelRefCell
{
    entry: gtk::Entry,
    entry_backlog: VecDeque<String>,
    entry_idx: Cell<usize>,
    entry_modified: Cell<bool>,
    backlog_scroll: gtk::ScrolledWindow,
    last_bottom: f64,
}

pub struct ChannelWidget
{
    name: String,
    pane: gtk::Box,
    backlog: gtk::TextView,
    empty: bool,
    refcell_data: Rc<RefCell<ChannelPanelRefCell>>,
}

impl ChannelWidget
{
    pub fn new(name: String) -> ChannelWidget
    {
        let pane = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let backlog_scroll = gtk::ScrolledWindow::new(None, None);
        let backlog = gtk::TextView::new();
        let entry = gtk::Entry::new();
        let mut entry_backlog = VecDeque::new();

        backlog_scroll.add(&backlog);
        pane.pack_start(&backlog_scroll, true, true, 0);
        pane.pack_start(&entry, false, false, 0);

        backlog.set_wrap_mode(gtk::WrapMode::WordChar);
        backlog.set_pixels_below_lines(5);
        backlog.set_left_margin(10);
        backlog.set_right_margin(10);

//        backlog_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Always);

        entry_backlog.push_front("".to_string());

        let refcell_data = Rc::new(RefCell::new(ChannelPanelRefCell
        {
            entry: entry, 
            entry_backlog: entry_backlog,
            entry_idx: Cell::new(0),
            entry_modified: Cell::new(false),
            backlog_scroll: backlog_scroll,
            last_bottom: 0.0,
        }));

        {
            let internal_copy = refcell_data.clone();
            backlog.connect_size_allocate(move |_, _| {
                let mut internal = internal_copy.borrow_mut();
                if let Some(adj) = internal.backlog_scroll.get_vadjustment()
                {
                    println!("vadj {}; aaa {}; up: {}; low {}; page {}", adj.get_value(), adj.get_upper() - adj.get_page_size(), adj.get_upper(), adj.get_lower(), adj.get_page_size());
                    let new_bottom = adj.get_upper() - adj.get_page_size();
                    if adj.get_value() == internal.last_bottom
                    {
                        adj.set_value(new_bottom);
                    }
                    internal.last_bottom = new_bottom;
                }
            });
        }

        {
            let internal_copy = refcell_data.clone();
            refcell_data.borrow().entry.connect_key_press_event(move |entry_ref, event_ref|
            {
                let mut internal = internal_copy.borrow_mut();
                let entry_idx = internal.entry_idx.get();
                if event_ref.get_keyval() == key::Up
                {
                    if entry_idx == 0
                    {
                        internal.entry_backlog[0] = entry_ref.get_text().unwrap();
                    }
                    if internal.entry_modified.get() == true
                    {
                        internal.entry_backlog.push_front(entry_ref.get_text().unwrap());
                        internal.entry_idx.set(0);
                        internal.entry_modified.set(false);
                    }
                    if let Some(text) = internal.entry_backlog.get(entry_idx + 1)
                    {
                        internal.entry_idx.set(entry_idx + 1);
                        internal.entry.set_text(text);
                        println!("entry_idx: {}", entry_idx);
                    }
                    return Inhibit(true);
                }
                else if event_ref.get_keyval() == key::Down
                {
                    if internal.entry_modified.get() == true
                    {
                        println!("{:?}", internal.entry_backlog);
                        internal.entry_backlog.push_front(entry_ref.get_text().unwrap());
                        internal.entry_backlog.push_front("".to_string());
                        internal.entry_idx.set(0);
                        internal.entry_modified.set(false);
                        entry_ref.set_text("");
                    }
                    else
                    {
                        if let Some(text) = internal.entry_backlog.get(entry_idx.saturating_sub(1))
                        {
                            internal.entry_idx.set(entry_idx.saturating_sub(1));
                            internal.entry.set_text(text);
                            println!("entry_idx: {}", entry_idx);
                        }
                    }
                    return Inhibit(true);
                }
                Inhibit(false)
            });
        }

        {
            let internal_copy = refcell_data.clone();
            refcell_data.borrow().entry.connect_delete_from_cursor(move |_, _, _|
            {
                println!("SUB");
                internal_copy.borrow_mut().entry_modified.set(true);
            });
        }

        {
            let internal_copy = refcell_data.clone();
            refcell_data.borrow().entry.connect_preedit_changed(move |_, _|
            {
                println!("ADD");
                internal_copy.borrow_mut().entry_modified.set(true);
            });
        }

        ChannelWidget
        {
            name: name,
            pane: pane,
            backlog: backlog,
            empty: true,
            refcell_data: refcell_data,
        }
    }

    pub fn get_pane(&self) -> &gtk::Box
    {
        &self.pane
    }

    pub fn println(&mut self, line: &str)
    {
        let buf = self.backlog.get_buffer().unwrap();
        let mut end = buf.get_end_iter();
        if !self.empty
        {
            buf.insert(&mut end, "\n");
        }
        else
        {
            self.empty = false;
        }
        buf.insert(&mut end, line);
    }

    pub fn print_message(&mut self, message: TwitchMessage, til: &mut TwitchImageLoader)
    {
        let buf = self.backlog.get_buffer().unwrap();
        let mut end = buf.get_end_iter();

        if !self.empty
        {
            buf.insert(&mut end, "\n");
        }
        else
        {
            self.empty = false;
        }

        match message
        {
            TwitchMessage::TwitchPrivmsg(privmsg) =>
            {
                for badge in privmsg.badges
                {
                    let receiver;

                    if badge.set == "subscriber"
                    {
                        receiver = til.get_subscriber_badge(badge.clone(), self.name[1..].into());
                    }
                    else
                    {
                        receiver = til.get_badge(badge.clone());
                    }

                    let badge_mark = gtk::TextMark::new(None, true);
                    buf.add_mark(&badge_mark, &end);
                    buf.insert(&mut end, " ");

                    let buf_clone = buf.clone();
                    let mark_clone = badge_mark.clone();
                    let async = move || {
                        if let Ok(bin) = receiver.try_recv()
                        {
                            println!("=====================LOADED BIN FOR THING");
                            let pbl = gdk_pixbuf::PixbufLoader::new();
                            pbl.loader_write(&bin).unwrap();
                            pbl.close().unwrap();
                            if let Some(pixbuf) = pbl.get_pixbuf()
                            {
                                let mut iter = buf_clone.get_iter_at_mark(&mark_clone);
                                buf_clone.insert_pixbuf(&mut iter, &pixbuf);
                            }
                            Continue(false)
                        }
                        else
                        {
                            Continue(true)
                        }
                    };

                    if let Continue(true) = async()
                    {
                        gtk::timeout_add(30, async);
                    }
                    else
                    {
                        end = buf.get_end_iter();
                    }
                }

                let msg = format!("<span foreground=\"#{:02x}{:02x}{:02x}\" font=\"bold\">{}</span>: ",
                                  privmsg.color.0,
                                  privmsg.color.1,
                                  privmsg.color.2,
                                  privmsg.name);
                buf.insert_markup(&mut end, &msg);

                let start_of_message_mark = gtk::TextMark::new(None, true);
                buf.add_mark(&start_of_message_mark, &end);

                buf.insert(&mut end, &privmsg.message);

                /* Generate marks first before deleting/inserting */
                let mut emote_vec = Vec::new();
                for emote in privmsg.emotes
                {
                    let receiver = til.get_emote(emote.id);

                    let mut range_marks = Vec::new();
                    for range in emote.ranges
                    {
                        let mut start = buf.get_iter_at_mark(&start_of_message_mark);
                        let mut end = buf.get_iter_at_mark(&start_of_message_mark);
                        start.forward_chars(range.0 as i32);
                        end.forward_chars(range.1 as i32 + 1);

                        let start_mark = gtk::TextMark::new(None, true);
                        buf.add_mark(&start_mark, &start);
                        let end_mark = gtk::TextMark::new(None, true);
                        buf.add_mark(&end_mark, &end);
                        range_marks.push((start_mark, end_mark));
                    }
                    emote_vec.push((receiver, range_marks));
                }

                for emote in emote_vec
                {
                    let buf_clone = buf.clone();
                    let range_marks_clone = emote.1;
                    let receiver = emote.0;
                    let async = move || {
                        if let Ok(bin) = receiver.try_recv()
                        {
                            println!("=====================LOADED BIN FOR EMOTE");
                            let pbl = gdk_pixbuf::PixbufLoader::new();
                            pbl.loader_write(&bin).unwrap();
                            pbl.close().unwrap();

                            if let Some(pixbuf) = pbl.get_pixbuf()
                            {
                                for range in &range_marks_clone
                                {
                                    let mut start = buf_clone.get_iter_at_mark(&range.0);
                                    let mut end = buf_clone.get_iter_at_mark(&range.1);
                                    buf_clone.delete(&mut start, &mut end);
                                    buf_clone.insert_pixbuf(&mut start, &pixbuf);
                                }
                            }
                            Continue(false)
                        }
                        else
                        {
                            Continue(true)
                        }
                    };

                    if let Continue(true) = async()
                    {
                        gtk::timeout_add(30, async);
                    }
                    /* Needed if we use end after this */
//                    else
//                    {
//                        end = buf.get_end_iter();
//                    }
                }

                buf.delete_mark(&start_of_message_mark);
            },
            TwitchMessage::TwitchEcho(mut privmsg, emote_sets) =>
            {
                for badge in &privmsg.badges
                {
                    let receiver;

                    if badge.set == "subscriber"
                    {
                        receiver = til.get_subscriber_badge(badge.clone(), self.name[1..].into());
                    }
                    else
                    {
                        receiver = til.get_badge(badge.clone());
                    }

                    let badge_mark = gtk::TextMark::new(None, true);
                    buf.add_mark(&badge_mark, &end);
                    buf.insert(&mut end, " ");

                    let buf_clone = buf.clone();
                    let mark_clone = badge_mark.clone();
                    let async = move || {
                        if let Ok(bin) = receiver.try_recv()
                        {
                            println!("=====================LOADED BIN FOR THING");
                            let pbl = gdk_pixbuf::PixbufLoader::new();
                            pbl.loader_write(&bin).unwrap();
                            pbl.close().unwrap();
                            if let Some(pixbuf) = pbl.get_pixbuf()
                            {
                                let mut iter = buf_clone.get_iter_at_mark(&mark_clone);
                                buf_clone.insert_pixbuf(&mut iter, &pixbuf);
                            }
                            Continue(false)
                        }
                        else
                        {
                            Continue(true)
                        }
                    };

                    if let Continue(true) = async()
                    {
                        gtk::timeout_add(30, async);
                    }
                    else
                    {
                        end = buf.get_end_iter();
                    }
                }

                let msg = format!("<span foreground=\"#{:02x}{:02x}{:02x}\" font=\"bold\">{}</span>: ",
                                  privmsg.color.0,
                                  privmsg.color.1,
                                  privmsg.color.2,
                                  privmsg.name);
                buf.insert_markup(&mut end, &msg);

                let start_of_message_mark = gtk::TextMark::new(None, true);
                buf.add_mark(&start_of_message_mark, &end);

                buf.insert(&mut end, &privmsg.message);

                for emote_set in emote_sets
                {
                    let receiver = til.get_emote_set(emote_set);

                    let buf_clone = buf.clone();
                    let privmsg_clone = privmsg.clone();
                    let mut til_clone = til.clone();
                    let start_of_message_mark_clone = start_of_message_mark.clone();
                    let mut async = move || {
                        if let Ok(emotes) = receiver.try_recv()
                        {
                            let mut end_clone = buf_clone.get_end_iter();
                            for emote in emotes
                            {
                                let mut emote_ranges = Vec::new();
                                for m in privmsg_clone.message.match_indices(&emote.code)
                                {
                                    let uh = privmsg_clone.message.char_indices()
                                                                  .collect::<Vec<_>>();
                                    println!("Found match {:?}", m);
                                    println!("PRINT UH");
                                    println!("{:?}", uh);
                                    let start_idx = privmsg_clone.message.char_indices()
                                                                         .enumerate()
                                                                         .filter(|i| (i.1).0 == m.0)
                                                                         .next()
                                                                         .map(|t| t.0)
                                                                         .unwrap_or(0);
                                    let code_len = emote.code.len();
                                    let end_idx = start_idx + code_len - 1;
                                    emote_ranges.push((start_idx, end_idx));
                                }

                                if !emote_ranges.is_empty()
                                {
                                    let image_receiver = til_clone.get_emote(emote.id);

                                    let mut range_marks = Vec::new();
                                    for range in emote_ranges
                                    {
                                        let mut start = buf_clone.get_iter_at_mark(&start_of_message_mark_clone);
                                        let mut end = buf_clone.get_iter_at_mark(&start_of_message_mark_clone);
                                        start.forward_chars(range.0 as i32);
                                        end.forward_chars(range.1 as i32 + 1);

                                        let start_mark = gtk::TextMark::new(None, true);
                                        buf_clone.add_mark(&start_mark, &start);
                                        let end_mark = gtk::TextMark::new(None, true);
                                        buf_clone.add_mark(&end_mark, &end);
                                        range_marks.push((start_mark, end_mark));
                                    }

                                    let buf_clone_clone = buf_clone.clone();
                                    let range_marks_clone = range_marks.clone();
                                    let image_async = move || {
                                        if let Ok(bin) = image_receiver.try_recv()
                                        {
                                            println!("=====================LOADED BIN FOR EMOTE");
                                            let pbl = gdk_pixbuf::PixbufLoader::new();
                                            pbl.loader_write(&bin).unwrap();
                                            pbl.close().unwrap();

                                            if let Some(pixbuf) = pbl.get_pixbuf()
                                            {
                                                for range in &range_marks_clone
                                                {
                                                    let mut start = buf_clone_clone.get_iter_at_mark(&range.0);
                                                    let mut end = buf_clone_clone.get_iter_at_mark(&range.1);
                                                    buf_clone_clone.delete(&mut start, &mut end);
                                                    buf_clone_clone.insert_pixbuf(&mut start, &pixbuf);
                                                }
                                            }
                                            Continue(false)
                                        }
                                        else
                                        {
                                            Continue(true)
                                        }
                                    };

                                    if let Continue(true) = image_async()
                                    {
                                        gtk::timeout_add(30, image_async);
                                    }
                                    else
                                    {
                                        end_clone = buf_clone.get_end_iter();
                                    }
                                }
                            }

                            Continue(false)
                        }
                        else
                        {
                            Continue(true)
                        }
                    };

                    if let Continue(true) = async()
                    {
                        gtk::timeout_add(30, async);
                    }
                    else
                    {
                        end = buf.get_end_iter();
                    }
                }
            },
            TwitchMessage::Unknown(raw) =>
            {
                buf.insert(&mut end, &raw);
            },
            _ =>
            {
                unreachable!();
            },
        }
    }

    pub fn on_text<F>(&mut self, callback: F) -> u64
        where F: Fn(String, String) + 'static
    {
        let internal_clone = self.refcell_data.clone();
        let name_clone = self.name.clone();
        self.refcell_data.borrow().entry.connect_activate(move |entry_ref|
            {
                let mut internal = internal_clone.borrow_mut();
                internal.entry_backlog[0] = entry_ref.get_text().unwrap();
                internal.entry_backlog.push_front("".to_string());
                internal.entry_idx.set(0);
                println!("entry_idx: 0");
                callback(name_clone.clone(), internal.entry.get_text().unwrap());
                internal.entry.set_text("");
            })
    }
}
