use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::sync::atomic::AtomicUsize;

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;

use rusoto_core::ByteStream;
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use tokio::io::AsyncReadExt;
use async_trait::async_trait;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Entry {
    pub id: u8,
    pub command: String,
    pub description: String,
}

impl Entry {
    pub fn new(command: String, description: String) -> Self {
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u8,
            command,
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub entries: Vec<Entry>,
}

#[async_trait]
pub trait Backend {
    async fn load_entries(&self) -> Collection;
    async fn save_entry_in(&mut self, mut collection: Collection, entry: Entry);
}

pub struct S3Storage {
    bucket: String,
    key: String,
    region: rusoto_core::Region,
}

impl S3Storage {
    pub fn new(bucket: String, key: String, region: rusoto_core::Region) -> Self {
        Self {
            bucket,
            key,
            region,
        }
    }
}

#[async_trait]
impl Backend for S3Storage {
    async fn load_entries(&self) -> Collection {
        let client = S3Client::new(self.region.to_owned());
        let request_input = rusoto_s3::GetObjectRequest {
            bucket: self.bucket.to_owned(),
            key: self.key.to_owned(),
            ..Default::default()
        };

        match client.get_object(request_input).await {
            Ok(output) => {
                // read the body of the response
                let body = output.body.unwrap();
                let mut reader = body.into_async_read();

                let mut contents = String::new();
                reader.read_to_string(&mut contents).await.unwrap();

                let mut collection = match serde_json::from_str::<Collection>(&contents.as_str()) {
                    Ok(mut collection) => {
                        // sort entries by id, increment the highest id by 1, and write to COUNTER
                        collection.entries.sort();
                        collection.entries.last().map(|entry| {
                            COUNTER.store(entry.id as usize + 1, std::sync::atomic::Ordering::Relaxed);
                        });
                        return collection
                    },
                    Err(e) => {
                        error!("unable to read entries: {}", e);
                        std::process::exit(exitcode::DATAERR);
                    }
                };
            }
            Err(_error) => {
                error!("getting object");
                Collection { entries: vec![] }
            }
        }

    }

    async fn save_entry_in(&mut self, mut collection: Collection, entry: Entry) {
        collection.entries.push(entry);

        let client = S3Client::new(self.region.to_owned());

        let entry_path = get_entry_path();
        let file = File::create(&entry_path).unwrap();

        match serde_json::to_writer_pretty(&file, &collection) {
            Ok(_) => {
                info!("successfully saved entry");
            }
            Err(e) => {
                error!("unable to update entries: {}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        }

        let mut contents = Vec::new();
        let mut open_file = File::open(entry_path).unwrap();
        open_file.read_to_end(&mut contents).unwrap();

        let mut request = PutObjectRequest::default();
        request.bucket = self.bucket.to_owned();
        request.key = self.key.to_owned();
        request.body = Some(ByteStream::from(contents));

        match client.put_object(request).await {
            Ok(output) => info!("{:?}", output),
            Err(error) => {
                error!("{:?}", error);
                std::process::exit(exitcode::SOFTWARE);
            }
        };
    }
}

pub struct LocalStorage {}

impl LocalStorage {
    pub fn new() -> Self {
        return LocalStorage {  };
    }
}

#[async_trait]
impl Backend for LocalStorage {
    async fn load_entries(&self) -> Collection {
        let entry_path = get_entry_path();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(entry_path)
            .unwrap();

        let mut contents = String::new();
        let size = file.read_to_string(&mut contents).unwrap();

        // return early if there's nothing in the file
        if size as i32 == 0 {
            return Collection { entries: vec![] };
        };

        let mut collection = match serde_json::from_str::<Collection>(&contents.as_str()) {
            Ok(mut collection) => {
                // sort entries by id, increment the highest id by 1, and write to COUNTER
                collection.entries.sort();
                collection.entries.last().map(|entry| {
                    COUNTER.store(entry.id as usize + 1, std::sync::atomic::Ordering::Relaxed);
                });
                return collection
            },
            Err(e) => {
                error!("unable to read entries: {}", e);
                std::process::exit(exitcode::DATAERR);
            }
        };
    }

    async fn save_entry_in(&mut self, mut collection: Collection, entry: Entry) {
        collection.entries.push(entry);

        // write Collection as json to file
        let entry_path = get_entry_path();
        let file = File::create(entry_path).unwrap();
        match serde_json::to_writer_pretty(file, &collection) {
            Ok(_) => {
                info!("successfully saved entry");
                std::process::exit(exitcode::OK);
            }
            Err(e) => {
                error!("unable to update entries: {}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        }
    }
}

fn get_entry_path() -> String {
    Path::new(&home_dir().unwrap())
        .join(".cargo/entries.json")
        .to_str()
        .unwrap()
        .to_string()
}
