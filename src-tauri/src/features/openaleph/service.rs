#![warn(unused_must_use)]
use ftm_types::generated::FtmEntity;
use serde::Deserialize;
use serde_envfile::from_file;
use serde_json::value::RawValue;
use std::path::PathBuf;
use tauri_plugin_http::reqwest::{self, header::AUTHORIZATION};

#[derive(Deserialize, Debug)]
struct EntityResult {
    status: String,
    results: Vec<Box<RawValue>>,
}

#[derive(Deserialize, Debug)]
struct ApiKey {
    api_key: String,
}

#[tauri::command]
pub async fn openaleph_search(query: String) -> Vec<String> {
    println!("Hello from openaleph search");
    // TODO use the otterly setting manager instead. This is a temporary workaround.
    let path = PathBuf::from(".env");
    let api_key = from_file::<ApiKey>(&path).expect("Failed to deserialize from file");
    // println!("{:?}", api_key);

    let client = reqwest::Client::builder()
        .user_agent("alephclient")
        .build()
        .unwrap();
    let res = client
        .get("https://search.openaleph.org/api/2/search")
        // TODO: sanitize!
        .query(&[("q", &query), ("limit", &"10".to_string())])
        .header(AUTHORIZATION, api_key.api_key)
        .send()
        .await;

    // This section is just a bit of playing around with the data-structures.
    // We probably don't need most of it later.
    let res = res.expect("reason");
    println!("{:?}", res.status()); // e.g. 200
    let entity_result = res.json::<EntityResult>().await.unwrap();
    println!("{:?}", entity_result.status);
    let mut v: Vec<FtmEntity> = Vec::new();
    for raw in &entity_result.results {
        let r = raw.get();
        println!("{:?}", &r);
        // this fails maybe due to version differences?
        let entity = FtmEntity::from_ftm_json(r).unwrap();
        match &entity {
            FtmEntity::Person(p) => {
                println!("{:?}, {:?}, {:?}", p.name, p.birth_date, p.nationality)
            }
            other => println!("{} {}", other.schema(), other.id()),
        };
        v.push(entity);
    }
    let mut rv: Vec<String> = Vec::new();
    for e in v {
        rv.push(FtmEntity::to_ftm_json(&e).unwrap());
    }

    // TODO: what kind of interface / result do we actually want here?
    rv
}
