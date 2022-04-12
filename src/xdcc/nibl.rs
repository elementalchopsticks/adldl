use super::{Bot, Pack};

use std::rc::Rc;
use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Error, Result};
use serde::Deserialize;
use ureq::{AgentBuilder, Request, Response};

#[derive(Deserialize)]
struct NiblSearchResponse {
    #[serde(flatten)]
    status: NiblStatus,

    #[serde(default)]
    content: Vec<NiblPack>,
}

#[derive(Deserialize)]
struct NiblBotsResponse {
    #[serde(flatten)]
    status: NiblStatus,

    #[serde(default)]
    content: Vec<NiblBot>,
}

#[derive(Deserialize)]
struct NiblStatus {
    status: String,
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NiblPack {
    bot_id: usize,
    number: usize,
    name: String,
    size: String,
    _sizekbits: usize,
    _episode_number: i64,
    _last_modified: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NiblBot {
    id: usize,
    name: String,
    _owner: String,
    _pack_size: usize,
}

pub fn search(query: &str, episode: Option<usize>) -> Result<Vec<Pack>> {
    let agent = AgentBuilder::new().timeout(Duration::from_secs(10)).build();

    let mut req = agent
        .get("https://api.nibl.co.uk/nibl/search")
        .query("query", query);

    if let Some(ep) = episode {
        req = req.query("episodeNumber", &ep.to_string());
    }

    let search_resp: NiblSearchResponse = send_request(req)?.into_json()?;
    check_status(&search_resp.status)?;

    let bots_resp: NiblBotsResponse =
        send_request(agent.get("https://api.nibl.co.uk/nibl/bots"))?.into_json()?;
    check_status(&bots_resp.status)?;

    let bot_index: HashMap<usize, Rc<Bot>> = bots_resp
        .content
        .into_iter()
        .map(|bot| (bot.id, Rc::new(Bot { name: bot.name })))
        .collect();

    let mut packs = Vec::with_capacity(search_resp.content.len());
    for nibl_pack in search_resp.content.into_iter() {
        let bot = Rc::clone(bot_index.get(&nibl_pack.bot_id).unwrap());
        packs.push(Pack {
            bot,
            number: nibl_pack.number,
            name: nibl_pack.name,
            size: nibl_pack.size,
        });
    }

    Ok(packs)
}

// Don't want ureq to mess with errors
fn send_request(request: Request) -> Result<Response> {
    match request.call() {
        Ok(response) => Ok(response),
        Err(ureq::Error::Status(_code, response)) => Ok(response),
        Err(error) => Err(Error::from(error)),
    }
}

fn check_status(status: &NiblStatus) -> Result<()> {
    if status.status != "OK" {
        Err(anyhow!("NIBL error: {}", status.message))
    } else {
        Ok(())
    }
}
