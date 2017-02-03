use twitch_message::{TwitchBadge, TwitchEmote};

use twitch_api::TwitchApi;
use twitch_api::model::emoticon::Emoticon;

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Read;

#[derive(Clone,Debug)]
enum ImageMessage
{
    GetBadge(TwitchBadge, Sender<Vec<u8>>),
    GetSubBadge((String, TwitchBadge), Sender<Vec<u8>>),
    GetEmote(u64, Sender<Vec<u8>>),
    GetEmoteSet(u64, Sender<Vec<TwitchEmote>>),
}

#[derive(Clone,Debug)]
pub struct TwitchImageLoader
{
    badge_url_cache: Arc<Mutex<HashMap<TwitchBadge, String>>>,
    sub_badge_url_cache: Arc<Mutex<HashMap<(String, TwitchBadge), String>>>,
    url_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    emote_cache: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
    emote_set_cache: Arc<Mutex<HashMap<u64, Vec<TwitchEmote>>>>,
    sender: Sender<ImageMessage>,
}

impl TwitchImageLoader
{
    pub fn new(client_id: &str) -> TwitchImageLoader
    {
        let tapi = TwitchApi::new(client_id);
        let (tx, rx) = mpsc::channel();
        let badge_url_cache = Arc::new(Mutex::new(HashMap::<TwitchBadge, String>::new()));
        let sub_badge_url_cache = Arc::new(Mutex::new(HashMap::<(String, TwitchBadge), String>::new()));
        let url_cache = Arc::new(Mutex::new(HashMap::<String, Vec<u8>>::new()));
        let emote_cache = Arc::new(Mutex::new(HashMap::<u64, Vec<u8>>::new()));
        let emote_set_cache = Arc::new(Mutex::new(HashMap::<u64, Vec<TwitchEmote>>::new()));

        let badge_url_cache_clone = badge_url_cache.clone();
        let sub_badge_url_cache_clone = sub_badge_url_cache.clone();
        let url_cache_clone = url_cache.clone();
        let emote_cache_clone = emote_cache.clone();
        let emote_set_cache_clone = emote_set_cache.clone();
        thread::spawn(move || {
            loop
            {
                match rx.recv()
                {
                    Ok(msg) =>
                    {
                        match msg
                        {
                            ImageMessage::GetBadge(badge, tx) =>
                            {
                                /* If badge in badge->url cache
                                 *     if url in url->bin cache
                                 *         callback(badge, bin)
                                 *     else
                                 *         get url
                                 *         add to url->bin cache
                                 *         callback(badge, bin)
                                 * else
                                 *     get badges
                                 *     add badges to badge->url cache
                                 *     get url
                                 *     add url to url->bin cache
                                 *     callback(badge, bin)
                                 */
                                let mut badge_url_cache = badge_url_cache_clone.lock().unwrap();
                                let mut url_cache = url_cache_clone.lock().unwrap();

                                if let Some(url) = badge_url_cache.get(&badge)
                                {
                                    if let Some(bin) = url_cache.get(url)
                                    {
                                        tx.send(bin.clone()).unwrap();
                                        continue;
                                    }
                                    let response_opt = tapi.make_raw_request(url);
                                    if let Ok(mut response) = response_opt
                                    {
                                        let mut buffer = Vec::new();
                                        response.read_to_end(&mut buffer).unwrap();
                                        tx.send(buffer.clone()).unwrap();
                                        url_cache.insert(url.clone(), buffer);
                                        continue;
                                    }
                                }
                                println!("GETTING BADGES");
                                let badge_sets = tapi.get_global_badges();
                                if let Ok(badge_sets) = badge_sets
                                {
                                    for set in badge_sets.badge_sets.keys()
                                    {
                                        let badge_set = badge_sets.badge_sets.get(set).unwrap();
                                        for version in badge_set.versions.keys()
                                        {
                                            let badge = TwitchBadge {
                                                set: set.clone(),
                                                version: version.clone(),
                                            };
                                            let url = badge_set.versions
                                                               .get(version)
                                                               .unwrap()
                                                               .image_url_1x
                                                               .clone();
                                            println!("BADGE {:?}, URL {}", badge, url);
                                            badge_url_cache.insert(badge, url);
                                        }
                                    }
                                }
                                if let Some(url) = badge_url_cache.get(&badge)
                                {
                                    let response_opt = tapi.make_raw_request(url);
                                    if let Ok(mut response) = response_opt
                                    {
                                        let mut buffer = Vec::new();
                                        println!("GOT BADGE DATA FOR URL {:?}", url);
                                        response.read_to_end(&mut buffer).unwrap();
                                        tx.send(buffer.clone()).unwrap();
                                        url_cache.insert(url.clone(), buffer);
                                    }
                                }
                            },
                            ImageMessage::GetSubBadge(badge, tx) =>
                            {
                                /* If badge in badge->url cache
                                 *     if url in url->bin cache
                                 *         callback(badge, bin)
                                 *     else
                                 *         get url
                                 *         add to url->bin cache
                                 *         callback(badge, bin)
                                 * else
                                 *     get badges
                                 *     add badges to badge->url cache
                                 *     get url
                                 *     add url to url->bin cache
                                 *     callback(badge, bin)
                                 */
                                let mut sub_badge_url_cache = sub_badge_url_cache_clone.lock().unwrap();
                                let mut url_cache = url_cache_clone.lock().unwrap();

                                if let Some(url) = sub_badge_url_cache.get(&badge)
                                {
                                    if let Some(bin) = url_cache.get(url)
                                    {
                                        tx.send(bin.clone()).unwrap();
                                        continue;
                                    }
                                    let response_opt = tapi.make_raw_request(url);
                                    if let Ok(mut response) = response_opt
                                    {
                                        let mut buffer = Vec::new();
                                        response.read_to_end(&mut buffer).unwrap();
                                        tx.send(buffer.clone()).unwrap();
                                        url_cache.insert(url.clone(), buffer);
                                        continue;
                                    }
                                }
                                println!("GETTING BADGES");
                                let user = tapi.get_user(&badge.0).unwrap().users.get(0).unwrap().id;
                                let badge_sets = tapi.get_subscriber_badges(user);
                                if let Ok(badge_sets) = badge_sets
                                {
                                    if let Some(subscriber_set) = badge_sets.badge_sets.get("subscriber")
                                    {
                                        for version in subscriber_set.versions.keys()
                                        {
                                            let sub_badge = TwitchBadge {
                                                set: "subscriber".into(),
                                                version: version.clone(),
                                            };
                                            let url = subscriber_set.versions
                                                               .get(version)
                                                               .unwrap()
                                                               .image_url_1x
                                                               .clone();
                                            println!("BADGE {:?}, URL {}", sub_badge, url);
                                            sub_badge_url_cache.insert((badge.0.clone(), sub_badge), url);
                                        }
                                    }
                                }
                                if let Some(url) = sub_badge_url_cache.get(&badge)
                                {
                                    let response_opt = tapi.make_raw_request(url);
                                    if let Ok(mut response) = response_opt
                                    {
                                        let mut buffer = Vec::new();
                                        println!("GOT BADGE DATA FOR URL {:?}", url);
                                        response.read_to_end(&mut buffer).unwrap();
                                        tx.send(buffer.clone()).unwrap();
                                        url_cache.insert(url.clone(), buffer);
                                    }
                                }
                            },
                            ImageMessage::GetEmote(emote_id, tx) =>
                            {
                                /* If emote in emote->bin cache
                                 *     send(bin)
                                 * else
                                 *     get emote image
                                 *     add emote to emote->bin cache
                                 *     send(bin)
                                 */
                                let mut emote_cache = emote_cache_clone.lock().unwrap();

                                if let Some(bin) = emote_cache.get(&emote_id)
                                {
                                    tx.send(bin.clone()).unwrap();
                                    continue;
                                }

                                println!("GETTING EMOTE {}", emote_id);
                                let emote_image = tapi.get_emoticon_image(emote_id, 1).unwrap();
                                tx.send(emote_image.clone()).unwrap();
                                emote_cache.insert(emote_id, emote_image);
                            },
                            ImageMessage::GetEmoteSet(emote_set, tx) =>
                            {
                                /* If emote in emote->bin cache
                                 *     send(bin)
                                 * else
                                 *     get emote image
                                 *     add emote to emote->bin cache
                                 *     send(bin)
                                 */
                                let mut emote_set_cache = emote_set_cache_clone.lock().unwrap();

                                if let Some(set) = emote_set_cache.get(&emote_set)
                                {
                                    tx.send(set.clone()).unwrap();
                                    continue;
                                }

                                println!("GETTING EMOTE SET {}", emote_set);
                                let emoticon_set = tapi.get_emoticons(vec![emote_set]).unwrap();
                                let twitch_emotes: Vec<_> = emoticon_set.emoticon_sets
                                                                        .values()
                                                                        .flat_map(|v| v.iter())
                                                                        .map(|e| TwitchEmote {
                                                                            id: e.id,
                                                                            code: e.code.clone(),
                                                                        })
                                                                        .collect();
                                tx.send(twitch_emotes.clone()).unwrap();
                                emote_set_cache.insert(emote_set, twitch_emotes);
                            },
                        }
                    },
                    Err(_) => break,
                }
            }
        });

        TwitchImageLoader
        {
            badge_url_cache: badge_url_cache,
            sub_badge_url_cache: sub_badge_url_cache,
            url_cache: url_cache,
            emote_cache: emote_cache,
            emote_set_cache: emote_set_cache,
            sender: tx,
        }
    }

    pub fn get_badge(&mut self, badge: TwitchBadge) -> Receiver<Vec<u8>>
    {
        let (tx, rx) = mpsc::channel();
        let badge_url_cache = self.badge_url_cache.lock().unwrap();
        if let Some(url) = badge_url_cache.get(&badge)
        {
            println!("FOUND BADGE AND GOT URL {}", url);
            let url_cache = self.url_cache.lock().unwrap();
            if let Some(bin) = url_cache.get(url)
            {
                println!("FOUND BIN FROM THAT URL");
                tx.send(bin.clone()).unwrap();
            }
            else
            {
                self.sender.send(ImageMessage::GetBadge(badge, tx)).unwrap();
            }
        }
        else
        {
            println!("FOUND NOTHING");
            self.sender.send(ImageMessage::GetBadge(badge, tx)).unwrap();
        }
        rx
    }

    pub fn get_subscriber_badge(&mut self, badge: TwitchBadge, chan: String) -> Receiver<Vec<u8>>
    {
        let (tx, rx) = mpsc::channel();
        let sub_badge_url_cache = self.sub_badge_url_cache.lock().unwrap();
        let tuple = (chan, badge);
        if let Some(url) = sub_badge_url_cache.get(&tuple)
        {
            println!("FOUND BADGE AND GOT URL {}", url);
            let url_cache = self.url_cache.lock().unwrap();
            if let Some(bin) = url_cache.get(url)
            {
                println!("FOUND BIN FROM THAT URL");
                tx.send(bin.clone()).unwrap();
            }
            else
            {
                self.sender.send(ImageMessage::GetSubBadge(tuple, tx)).unwrap();
            }
        }
        else
        {
            println!("FOUND NOTHING");
            self.sender.send(ImageMessage::GetSubBadge(tuple, tx)).unwrap();
        }
        rx
    }

    pub fn get_emote(&mut self, emote_id: u64) -> Receiver<Vec<u8>>
    {
        let (tx, rx) = mpsc::channel();
        let emote_cache = self.emote_cache.lock().unwrap();
        if let Some(bin) = emote_cache.get(&emote_id)
        {
            tx.send(bin.clone()).unwrap();
        }
        else
        {
            self.sender.send(ImageMessage::GetEmote(emote_id, tx)).unwrap();
        }
        rx
    }

    pub fn get_emote_set(&mut self, emote_set: u64) -> Receiver<Vec<TwitchEmote>>
    {
        let (tx, rx) = mpsc::channel();
        let emote_set_cache = self.emote_set_cache.lock().unwrap();
        if let Some(set) = emote_set_cache.get(&emote_set)
        {
            tx.send(set.clone()).unwrap();
        }
        else
        {
            self.sender.send(ImageMessage::GetEmoteSet(emote_set, tx)).unwrap();
        }
        rx
    }
}
