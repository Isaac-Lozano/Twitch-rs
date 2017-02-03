use twitch_message::{TwitchMessage, TwitchPrivmsg, TwitchUserState, UserColor};
use twitch_image_loader::TwitchImageLoader;

use ui::main_window::MainWindow;

use twitch_chat::client::{ChatClient, ChatSender, TwitchReceiver, TwitchSender, ClientError, ClientResult};
use twitch_chat::message::Message;

use gtk::prelude::*;
use gtk;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::sync::mpsc::{self, Receiver};
use std::collections::HashMap;

const CLIENT_ID: &'static str = "8t59b442f4twnhw5ur8vw7vf5muoauf";

pub struct Client
{
    sender: ChatSender,
    receiver: Receiver<ClientResult<Message>>,
}

pub struct TwitchRS
{
    window: Rc<RefCell<MainWindow>>,
    client: Rc<RefCell<Option<Client>>>,
    twitch_loader: Rc<RefCell<TwitchImageLoader>>,
    global_user_state: Rc<RefCell<TwitchUserState>>,
    channel_user_state: Rc<RefCell<HashMap<String, TwitchUserState>>>,
}

impl TwitchRS
{
    pub fn new() -> TwitchRS
    {
        gtk::init().expect("Could not init gtk");

        let init_user_state = TwitchUserState {
            badges: Vec::new(),
            color: UserColor(0, 0, 0),
            display_name: String::new(),
            emote_sets: Vec::new(),
            user_id: 0,
            user_type: (),
        };

        let mut trs = TwitchRS
        {
            window: Rc::new(RefCell::new(MainWindow::new())),
            client: Rc::new(RefCell::new(None)),
            twitch_loader: Rc::new(RefCell::new(TwitchImageLoader::new(CLIENT_ID))),
            global_user_state: Rc::new(RefCell::new(init_user_state)),
            channel_user_state: Rc::new(RefCell::new(HashMap::new())),
        };

        trs.setup_callbacks();

        return trs;
    }

    fn setup_callbacks(&mut self)
    {
        {
            struct Env
            {
                window_clone: Rc<RefCell<MainWindow>>,
                client_clone: Rc<RefCell<Option<Client>>>,
                twitch_loader_clone: Rc<RefCell<TwitchImageLoader>>,
                global_user_state_clone: Rc<RefCell<TwitchUserState>>,
                channel_user_state_clone: Rc<RefCell<HashMap<String, TwitchUserState>>>,
            }
            let env = Env
            {
                window_clone: self.window.clone(),
                client_clone: self.client.clone(),
                twitch_loader_clone: self.twitch_loader.clone(),
                global_user_state_clone: self.global_user_state.clone(),
                channel_user_state_clone: self.channel_user_state.clone(),
            };

            fn on_text_callback(tab_name: String, text: String, env: &mut Env)
            {
                let mut window = env.window_clone.borrow_mut();
                let mut client = env.client_clone.borrow_mut();
                let mut twitch_loader = env.twitch_loader_clone.borrow_mut();
                let global_user_state = env.global_user_state_clone.borrow_mut();
                let channel_user_state = env.channel_user_state_clone.borrow_mut();

                if text.starts_with("/")
                {
                    let args: Vec<_> = text[1..].split(' ').collect();
                    if let Some(cmd) = args.get(0)
                    {
                        match *cmd
                        {
                            "join" =>
                            {
                                if let Some(ref mut twitch_client) = *client
                                {
                                    if let Some(channel) = args.get(1)
                                    {
                                        let inner_env = Env
                                        {
                                            window_clone: env.window_clone.clone(),
                                            client_clone: env.client_clone.clone(),
                                            twitch_loader_clone: env.twitch_loader_clone.clone(),
                                            global_user_state_clone: env.global_user_state_clone.clone(),
                                            channel_user_state_clone: env.channel_user_state_clone.clone(),
                                        };
                                        window.add_channel(String::from(*channel), on_text_callback, inner_env);
                                        twitch_client.sender.send_join(channel).unwrap();
                                    }
                                }
                            },
                            "ban" | "timeout" | "color" | "host" | "unban" =>
                            {
                                if let Some(ref mut twitch_client) = *client
                                {
                                    twitch_client.sender.send_message(&tab_name, &text).unwrap();
                                    let user_state = channel_user_state.get(&tab_name)
                                                                       .unwrap_or(&global_user_state);
                                    let echo_message = TwitchMessage::TwitchEcho (
                                        TwitchPrivmsg {
                                            name: user_state.display_name.clone(),
                                            emotes: Vec::new(),
                                            badges: user_state.badges.clone(),
                                            color: user_state.color,
                                            to: tab_name.clone(),
                                            message: text.clone(),
                                        },
                                        user_state.emote_sets.clone()
                                    );
                                    window.channel_print_message(tab_name, echo_message, &mut twitch_loader);
                                }
                            }
                            _ => {},
                        }
                    }
                }
                else if &tab_name != "status"
                {
                    if let Some(ref mut twitch_client) = *client
                    {
                        twitch_client.sender.send_message(&tab_name, &text).unwrap();
                        let user_state = channel_user_state.get(&tab_name)
                                                           .unwrap_or(&global_user_state);
                        let echo_message = TwitchMessage::TwitchEcho (
                            TwitchPrivmsg {
                                name: user_state.display_name.clone(),
                                emotes: Vec::new(),
                                badges: user_state.badges.clone(),
                                color: user_state.color,
                                to: tab_name.clone(),
                                message: text.clone(),
                            },
                            user_state.emote_sets.clone()
                        );
                        window.channel_print_message(tab_name, echo_message, &mut twitch_loader);
                    }
                }
            };
            self.window.borrow_mut().on_status_text(on_text_callback, env);
        }

        {
            let client_clone = self.client.clone();
            self.window.borrow_mut().on_login(move |auth| {
                let mut client = client_clone.borrow_mut();
                let mut chat_client = ChatClient::connect().unwrap();
                chat_client.send_authenticate(auth).unwrap();
                let (sender, mut receiver) = chat_client.split();
                let (tx, rx) = mpsc::channel();
    
                thread::spawn(move || {
                    loop
                    {
                        let result = receiver.get_message();
                        if let Err(ClientError::WebSocketError(_)) = result
                        {
                            tx.send(result).unwrap();
                            return;
                        }
                        tx.send(result).unwrap();
                    }
                });
                *client = Some(Client {
                    sender: sender,
                    receiver: rx,
                });
            });
        }

        {
            let window_clone = self.window.clone();
            let client_clone = self.client.clone();
            let twitch_loader_clone = self.twitch_loader.clone();
            let global_user_state_clone = self.global_user_state.clone();
            let channel_user_state_clone = self.channel_user_state.clone();

            gtk::timeout_add(30, move ||
                {
                    let mut window = window_clone.borrow_mut();
                    let mut client = client_clone.borrow_mut();
                    let mut twitch_loader = twitch_loader_clone.borrow_mut();
                    let mut global_user_state = global_user_state_clone.borrow_mut();
                    let mut channel_user_state = channel_user_state_clone.borrow_mut();

                    if let Some(ref mut twitch_client) = *client
                    {
                        while let Ok(msg) = twitch_client.receiver.try_recv()
                        {
                            if let Err(e) = msg
                            {
                                match e
                                {
                                    ClientError::WebSocketError(_) =>
                                    {
                                        window.status_log_line("Disconnected from chat");
                                        //return Continue(false);
                                    },
                                    _ =>
                                    {
                                        println!("Error: {}", e);
                                    }
                                }
                            }
                            else if let Ok(message) = msg
                            {
                                let command = message.command.clone();
                                match command.as_str()
                                {
                                    "PRIVMSG" =>
                                    {
                                        let chan_opt = message.args.get(0)
                                                                   .map(|s| s.clone());
                                        if let Some(chan_name) = chan_opt
                                        {
                                            window.channel_print_message(chan_name.clone(), message.into(), &mut twitch_loader);
                                        }
                                    },
                                    "NOTICE" =>
                                    {
                                        let chan_opt = message.args.get(0)
                                                                   .map(|s| s.clone());
                                        if let Some(chan_name) = chan_opt
                                        {
                                            if let Some(notice_message) = message.args.get(1)
                                            {
                                                window.channel_print_line(chan_name.clone(), &notice_message);
                                            }
                                        }
                                    },
                                    "PING" =>
                                    {
                                        if let Some(value) = message.args.get(0)
                                        {
                                            twitch_client.sender.send_raw(&format!("PONG :{}", value)).unwrap();
                                        }
                                    },
                                    "GLOBALUSERSTATE" =>
                                    {
                                        let msg: TwitchMessage = message.into();
                                        /* Garenteed */
                                        if let TwitchMessage::TwitchGlobalUserState(global_state) = msg
                                        {
                                            *global_user_state = global_state;
                                        }
                                    },
                                    "USERSTATE" =>
                                    {
                                        let msg: TwitchMessage = message.into();
                                        /* Garenteed */
                                        if let TwitchMessage::TwitchUserState(chan, user_state) = msg
                                        {
                                            channel_user_state.insert(chan, user_state);
                                        }
                                    },
                                    _ =>
                                    {
                                        window.status_log_line(&message.raw);
                                    }
                                }
                            }
                        }
                    }
                    Continue(true)
                }
            );
        }
    }

    pub fn run(&mut self)
    {
        gtk::main();
    }
}
