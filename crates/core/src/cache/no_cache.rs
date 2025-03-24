use serde::{Serialize, de::DeserializeOwned};

use super::{Cache, EXPIRE_AT, TimedObject};

pub const EPOCH_YEAR: u32 = 2025;

#[derive(Clone)]
pub struct NoCache {
    pub client: Option<u32>,
}

impl Cache for NoCache {
    type Item = String;
    type Client = Option<u32>;

    async fn new() -> Self {
        Self { client: None }
    }

    async fn get_con(&self) -> Self::Client {
        None
    }

    async fn get(&self, id: Self::Item) -> Option<String> {
        None
    }

    async fn set(&self, id: Self::Item, content: Self::Item) -> bool {
        true
    }

    async fn update(&self, id: Self::Item, content: Self::Item) -> bool {
        true
    }

    async fn remove(&self, id: Self::Item) -> bool {
        true
    }

    async fn remove_starting_with(&self, id: Self::Item) -> bool {
        true
    }

    async fn incr(&self, id: Self::Item) -> bool {
        true
    }

    async fn decr(&self, id: Self::Item) -> bool {
        true
    }

    async fn get_timed<T: Serialize + DeserializeOwned>(
        &self,
        id: Self::Item,
    ) -> Option<TimedObject<T>> {
        None
    }

    async fn set_timed<T: Serialize + DeserializeOwned>(&self, id: Self::Item, content: T) -> bool {
        None
    }
}
