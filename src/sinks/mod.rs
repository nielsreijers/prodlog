use serde::Deserialize;
use uuid::Uuid;
use crate::model::CaptureV2_4;


pub mod sqlite;

pub trait Sink: Send + Sync {
    fn add_entry(&self, capture: &CaptureV2_4) -> Result<(), std::io::Error>;
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
    fn create_task(&self, name: &str) -> Result<i64, std::io::Error>;
    fn get_all_tasks(&self) -> Result<Vec<crate::model::Task>, std::io::Error>;
    fn update_task_name(&self, task_id: i64, name: &str) -> Result<(), std::io::Error>;
    fn assign_entries_to_task(&self, entry_uuids: &[String], task_id: Option<i64>) -> Result<(), std::io::Error>;
    fn get_active_task(&self) -> Result<Option<i64>, std::io::Error>;
    fn set_active_task(&self, task_id: Option<i64>) -> Result<(), std::io::Error>;
}




