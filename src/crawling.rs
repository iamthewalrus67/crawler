use std::{ops::Deref, num::NonZeroU32, sync::Arc};

use crossbeam_channel::{Receiver, Sender};
use futures::{StreamExt, Future};
use governor::{state::{NotKeyed, InMemoryState}, clock::{QuantaClock, QuantaInstant}, middleware::NoOpMiddleware};
use reqwest::{Request, Response};

use crate::parsing::{parse, ParseArguments, ParsedData};

struct Agent {
    thread: std::thread::JoinHandle<()>,
}

impl Agent {
    fn new(name: String, url_receiver: Receiver<Vec<String>>, data_sender: Sender<Result<ParsedData, String>>, client: reqwest::Client, rate_limiter: Arc<governor::RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>>) -> Self {
        let thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                                    .enable_io()
                                    .enable_time()
                                    .build()
                                    .unwrap();

            rt.block_on(async move {
                loop {
                    let urls = match url_receiver.recv() {
                        Ok(v) => futures::stream::iter(v),
                        Err(_) => break,
                    };

                    urls.for_each_concurrent(Some(1), |url| {
                        let url_clone = url.clone();
                        let data_sender = data_sender.clone();
                        let client = client.clone();
                        let rate_limiter = Arc::clone(&rate_limiter);
                        async move {
                            match parse(ParseArguments {
                                                        page_url: url_clone.clone(),
                                                        base_url: url_clone,
                                                    },
                                                    &client,
                                                    &rate_limiter).await {
                                Ok(parsed_data) => {
                                    data_sender.send(Ok(parsed_data)).unwrap();
                                },
                                Err(url) => data_sender.send(Err(url)).unwrap(),
                            };
                        }
                    }).await;
                }
            });
        });

        Self {thread}
    }
}

pub struct AgentManager {
    agents: Vec<Agent>,
    pub url_sender: Sender<Vec<String>>,
    pub data_receiver: Receiver<Result<ParsedData, String>>,
}

impl AgentManager {
    pub fn new(agents_num: usize, name: String) -> Self {
        let (url_sender, url_receiver) = crossbeam_channel::unbounded();
        let (data_sender, data_receiver) = crossbeam_channel::unbounded();

        let client = reqwest::ClientBuilder::new()
            .user_agent(name.clone())
            .build()
            .expect("Could not create a reqwest client");

        let q = governor::Quota::per_minute(NonZeroU32::new(30).unwrap());
    
        let r = std::sync::Arc::new(governor::RateLimiter::direct(q));
        
        let mut agents = vec![];
        for _ in 0..agents_num {
            agents.push(Agent::new(name.clone(), url_receiver.clone(), data_sender.clone(), client.clone(), Arc::clone(&r)));
        }

        Self {agents, url_sender, data_receiver}
    }
}
