use ui::channel::ChannelWidget;
use twitch_image_loader::TwitchImageLoader;

use twitch_message::TwitchMessage;

use twitch_chat::auth::Auth;

use gdk;
use gtk;
use gtk::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct MainWindow
{
    notebook: gtk::Notebook,
    channels: HashMap<String, ChannelWidget>,
    status: ChannelWidget,
    login_callback: Rc<RefCell<Option<Box<Fn(Option<Auth>) + 'static>>>>,
}

impl MainWindow
{
    pub fn new() -> MainWindow
    {
        let css = gtk::CssProvider::new();
        match css.load_from_path("resources/style.css")
        {
            Ok(_) =>
                gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION),
            Err(_) =>
                println!("Error: Could not load style sheet."),
        }

        let status = ChannelWidget::new(String::from("Status"));

        let menu_bar = gtk::MenuBar::new();
        let login = gtk::MenuItem::new_with_label("Login");
        let login_menu = gtk::Menu::new();
        let with_auth = gtk::MenuItem::new_with_label("With Auth");
        let anonymous = gtk::MenuItem::new_with_label("Anonymous");
        login_menu.append(&with_auth);
        login_menu.append(&anonymous);
        login.set_submenu(Some(&login_menu));
        menu_bar.append(&login);

        let notebook = gtk::Notebook::new();
        notebook.append_page(status.get_pane(), Some(&gtk::Label::new(Some("Status"))));
        notebook.set_scrollable(true);

        let main_pane = gtk::Box::new(gtk::Orientation::Vertical, 5);
        main_pane.pack_start(&menu_bar, false, false, 0);
        main_pane.pack_start(&notebook, true, true, 0);

        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        win.set_title("Twitch chat (By OnVar)");
        win.set_default_size(300, 500);
        win.add(&main_pane);
        
        let login_callback: Rc<RefCell<Option<Box<Fn(Option<Auth>) + 'static>>>> = Rc::new(RefCell::new(None));

        {
            let login_callback_clone = login_callback.clone();
            with_auth.connect_activate(move |_| {
                let auth_dialog = gtk::Dialog::new_with_buttons::<gtk::Window>(Some("Login"),
                                                                               None,
                                                                               gtk::DIALOG_DESTROY_WITH_PARENT | gtk::DIALOG_MODAL,
                                                                               &[("login", 0)]);
                let content_area = auth_dialog.get_content_area();

                let content_grid = gtk::Grid::new();

                let username_label = gtk::Label::new(Some("Username"));
                username_label.set_padding(3, 0);
                let username_entry = gtk::Entry::new();
                content_grid.attach(&username_label, 0, 0, 1, 1);
                content_grid.attach(&username_entry, 1, 0, 1, 1);

                let oauth_label = gtk::Label::new(Some("OAuth"));
                let oauth_entry = gtk::Entry::new();
                content_grid.attach(&oauth_label, 0, 1, 1, 1);
                content_grid.attach(&oauth_entry, 1, 1, 1, 1);

                content_area.pack_start(&content_grid, false, false, 0);

                {
                    let auth_dialog_clone = auth_dialog.clone();
                    username_entry.connect_activate(move |_| {
                        auth_dialog_clone.response(0);
                    });
                }

                {
                    let auth_dialog_clone = auth_dialog.clone();
                    oauth_entry.connect_activate(move |_| {
                        auth_dialog_clone.response(0);
                    });
                }

                let username_entry_clone = username_entry.clone();
                let oauth_entry_clone = oauth_entry.clone();
                
                /* To prevent moving login_callback_clone out of this closure */
                let login_callback_clone_clone = login_callback_clone.clone();
                auth_dialog.connect_response(move |dialog_ref, response_id| {
                    if response_id == 0
                    {
                        if let Some(ref login_callback) = *login_callback_clone_clone.borrow_mut()
                        {
                            let username = username_entry_clone.get_text().unwrap();
                            let oauth = oauth_entry_clone.get_text().unwrap();
                            login_callback(Some(Auth::new(username, oauth)));
                        }
                    }
                    dialog_ref.close();
                });

                auth_dialog.show_all();
            });
        }

        {
            let login_callback_clone = login_callback.clone();
            anonymous.connect_activate(move |_| {
                if let Some(ref login_callback) = *login_callback_clone.borrow_mut()
                {
                    login_callback(None);
                }
            });
        }

        win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        win.show_all();

        MainWindow
        {
            notebook: notebook,
            channels: HashMap::new(),
            status: status,
            login_callback: login_callback,
        }
    }

    pub fn add_channel<F, T>(&mut self, name: String, callback: F, ctx: T)
        where F: Fn(String, String, &mut T) + 'static, T: 'static
    {
        let mut new_ch = ChannelWidget::new(name.clone());
        self.notebook.append_page(new_ch.get_pane(), Some(&gtk::Label::new(Some(&name.clone()))));
        self.notebook.set_tab_reorderable(new_ch.get_pane(), true);
        let ctx_rc = Rc::new(RefCell::new(ctx));
        let ctx_rc_clone = ctx_rc.clone();
        new_ch.on_text(move |name, text|
                       {
                           let mut ctx_rc = ctx_rc_clone.borrow_mut();
                           callback(name, text, &mut *ctx_rc);
                       });
        self.channels.insert(name, new_ch);
        self.notebook.show_all();
    }

    pub fn on_status_text<F, T>(&mut self, callback: F, ctx: T)
        where F: Fn(String, String, &mut T) + 'static, T: 'static
    {
        let ctx_rc = Rc::new(RefCell::new(ctx));
        let ctx_rc_clone = ctx_rc.clone();
        self.status.on_text(move |name, text|
                       {
                           let mut ctx_rc = ctx_rc_clone.borrow_mut();
                           callback(name, text, &mut *ctx_rc);
                       });
    }

    pub fn on_login<F>(&mut self, callback: F)
        where F: Fn(Option<Auth>) + 'static
    {
        *self.login_callback.borrow_mut() = Some(Box::new(callback));
    }

    pub fn status_log_line(&mut self, line: &str)
    {
        self.status.println(line);
    }

    pub fn channel_print_message(&mut self, channel: String, message: TwitchMessage, til: &mut TwitchImageLoader)
    {
        if let Some(channel) = self.channels.get_mut(&channel)
        {
            channel.print_message(message, til);
        }
    }

    pub fn channel_print_line(&mut self, channel: String, message: &str)
    {
        if let Some(channel) = self.channels.get_mut(&channel)
        {
            channel.println(message);
        }
    }
}
