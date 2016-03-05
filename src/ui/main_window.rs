extern crate gtk;

use ui::channel::ChannelWidget;
use gtk::traits::*;
use gtk::signal::Inhibit;

pub struct MainWindow
{
    win: gtk::Window,
    main_pane: gtk::Box,
    menu: gtk::Box,
    notebook: gtk::NoteBook,
    channels: Vec<ChannelWidget>,
    status: ChannelWidget,
}

impl MainWindow
{
    pub fn new() -> MainWindow
    {
        /* TODO: Error checking */
        let status = ChannelWidget::new(String::from("Status"));

        let menu = MainWindow::make_menu();
        let notebook = gtk::NoteBook::new().unwrap();
        notebook.append_page(status.get_pane(), Some(&gtk::Label::new(&"Status").unwrap()));

        let main_pane = gtk::Box::new(gtk::Orientation::Vertical, 5).unwrap();
        main_pane.pack_start(&menu, false, false, 0);
        main_pane.pack_start(&notebook, true, true, 0);

        let win = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
        win.set_default_size(300, 500);
        win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        win.add(&main_pane);

        win.show_all();

        MainWindow
        {
            win: win,
            main_pane: main_pane,
            menu: menu,
            notebook: notebook,
            channels: Vec::new(),
            status: status,
        }
    }

    pub fn add_channel(&mut self, name: String)
    {
        let new_ch = ChannelWidget::new(name.clone());
        self.notebook.append_page(new_ch.get_pane(), Some(&gtk::Label::new(&name.clone()).unwrap()));
        self.channels.push(new_ch);
        self.notebook.show_all();
    }

    pub fn on_status_entry<F>(&mut self, callback: F) -> u64
        where F: Fn(gtk::Entry) + 'static
    {
        self.status.on_entry_activate(callback)
    }

    fn make_menu() -> gtk::Box
    {
        let menu = gtk::Box::new(gtk::Orientation::Horizontal, 0).unwrap();

        let test = gtk::Button::new_with_label("Test").unwrap();
        let test2 = gtk::Button::new_with_label("Test2").unwrap();

        menu.pack_start(&test, false, false, 0);
        menu.pack_start(&test2, false, false, 0);

        menu
    }
}
