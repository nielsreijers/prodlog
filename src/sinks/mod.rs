use serde::Deserialize;
use uuid::Uuid;
use crate::model::CaptureV2_4;


pub mod sqlite;

pub trait Sink: Send + Sync {
    fn add_entry(&mut self, capture: &CaptureV2_4) -> Result<(), std::io::Error>;
}

#[derive(Deserialize, Debug, Default)]
pub struct Filters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub host: Option<String>,
    pub search: Option<String>,
    pub search_content: Option<String>,
    pub show_noop: Option<bool>,
}

pub trait UiSource: Sink + Send + Sync {
    fn get_entries(&self, filters: &Filters) -> Result<Vec<CaptureV2_4>, std::io::Error>;
    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_4>, std::io::Error>;
}




