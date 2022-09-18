use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::sync::atomic::AtomicUsize;

use std::fs::File;
use std::io::Read;

static COUNTER: AtomicUsize = AtomicUsize::new(1);
static ENTRY_PATH: &str = "entries.json";

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
    entries: Vec<Entry>,
}

impl Collection {
    pub fn new() -> Self {
        let mut file = File::open(ENTRY_PATH).unwrap();

        let mut contents = String::new();
        let size = file.read_to_string(&mut contents).unwrap();

        if size as i32 == 0 {
            return Self { entries: vec![] };
        };

        let mut collection = match serde_json::from_str::<Collection>(&contents.as_str()) {
            Ok(collection) => collection,
            Err(e) => {
                error!("unable to read entries: {}", e);
                std::process::exit(exitcode::DATAERR);
            }
        };

        // sort entries by id, increment the highest id by 1, and write to COUNTER
        collection.entries.sort();
        collection.entries.last().map(|entry| {
            COUNTER.store(entry.id as usize + 1, std::sync::atomic::Ordering::Relaxed);
        });
        collection
    }

    pub fn save(&mut self, entry: Entry) {
        self.entries.push(entry);

        // write Collection as json to file
        let file = File::create(ENTRY_PATH).unwrap();
        match serde_json::to_writer_pretty(file, self) {
            Ok(_) => {
                // TODO info!("successfully saved entry: {:?}", entry);
                info!("successfully saved entry");
                std::process::exit(exitcode::OK);
            }
            Err(e) => {
                error!("unable to update entries: {}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        }
    }

    pub fn get_entries(&self) -> &Vec<Entry> {
        &self.entries
    }
}