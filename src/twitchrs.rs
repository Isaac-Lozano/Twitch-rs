use ui::main_window::MainWindow;
use twitch_chat::client::Client;

pub struct TwitchRS
{
    window: MainWindow,
    client: Client,
}

impl TwitchRS
{
    pub fn new() -> TwitchRS
    {
        let mut trs = TwitchRS
        {
            window: MainWindow::new(),
            client: Client::new(),
        };

        trs.window.on_status_entry(|_|
            {
                println!("TESTASDASDFSADF");
            });

        return trs;
    }
}

