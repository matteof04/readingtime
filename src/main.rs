/*
 * Copyright (c) 2024 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */

use std::{env, process::exit, sync::Arc};

use log::{error, info, trace, warn};
use readingtime::calculate_reading_time;
use regex::Regex;
use reqwest::Url;
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{MessageEntityKind, ReplyParameters},
};
use thiserror::Error;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();
    let bot_token = match env::var("BOT_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("BOT_TOKEN not set!");
            exit(1)
        }
    };
    let wpm = match env::var("WPM") {
        Ok(wpm) => {
            let wpm_parsed: Result<f32, _> = wpm.parse();
            match wpm_parsed {
                Ok(wpm) => wpm,
                Err(_) => {
                    warn!("Error in WPM environment variable parsing, defualting to 225.0");
                    225.0
                }
            }
        }
        Err(_) => 225.0,
    };
    info!("WPM: {wpm}");
    let wpm = Arc::new(wpm);
    let handler =
        Update::filter_message().endpoint(|bot: Bot, wpm: Arc<f32>, msg: Message| async move {
            if let Some(user) = &msg.from {
                trace!("New message from user with ID: {:?}", user.id);
            }
            if let Some(url) = extract_url(&msg) {
                match url {
                    Ok(u) => match get_content(u.to_owned()).await {
                        Ok(content) => {
                            let reading_time = calculate_reading_time(&content, *wpm);
                            bot.send_message(
                                msg.chat.id,
                                format!("Estimated reading time: {reading_time} minutes"),
                            )
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                        }
                        Err(e) => {
                            bot.send_message(msg.chat.id, format!("{e}"))
                                .reply_parameters(ReplyParameters::new(msg.id))
                                .await?;
                        }
                    },
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("{e}"))
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, format!("{}", ProcessError::UrlParsing))
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            respond(())
        });
    let bot = Bot::new(bot_token);
    info!("Starting the bot...");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![wpm])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(Debug, Error)]
enum ProcessError {
    #[error("Not a valid URL")]
    UrlParsing,
    #[error("The site content can't be fetched")]
    Request,
    #[error("The site content is not a valid HTML")]
    HtmlParsing,
}

fn extract_url(msg: &Message) -> Option<Result<Url, ProcessError>> {
    let mut urls: Vec<Result<Url, ProcessError>> = vec![];
    if let Some(entities) = msg.parse_entities() {
        let mut parsed_text_links: Vec<Result<Url, ProcessError>> = entities
            .into_iter()
            .filter_map(|e| {
                if let MessageEntityKind::TextLink { url } = e.kind() {
                    Some(Ok(url.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        urls.append(&mut parsed_text_links);
    }
    if let Some(entities) = msg.parse_caption_entities() {
        let mut parsed_caption_links: Vec<Result<Url, ProcessError>> = entities
            .into_iter()
            .filter_map(|e| {
                if let MessageEntityKind::TextLink { url } = e.kind() {
                    Some(Ok(url.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        urls.append(&mut parsed_caption_links);
    }
    let msg_text = msg.text().unwrap_or("");
    let mut parsed_msg_text = parse_url(msg_text);
    urls.append(&mut parsed_msg_text);
    let caption_text = msg.caption().unwrap_or("");
    let mut parsed_caption_text = parse_url(caption_text);
    urls.append(&mut parsed_caption_text);
    urls.into_iter().next()
}

fn parse_url(text: &str) -> Vec<Result<Url, ProcessError>> {
    Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)")
        .unwrap()
        .find_iter(text)
        .map(|m| m.as_str())
        .map(|s| Url::parse(s).map_err(|_| ProcessError::UrlParsing))
        .collect()
}

async fn get_content(url: Url) -> Result<String, ProcessError> {
    reqwest::get(url)
        .await
        .map_err(|_| ProcessError::Request)?
        .text()
        .await
        .map_err(|_| ProcessError::HtmlParsing)
}
