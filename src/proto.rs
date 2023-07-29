use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::{Number, Value};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexDataType {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub id: String,
    pub settings: Option<MemexDataTypeSettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexDataTypeSettings {
    pub options: Option<Vec<MemexDataTypeOption>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexDataTypeOption {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexItem {
    pub content_id: i64,
    pub content_type: String,
    pub content_repository_id: Value,
    pub id: i64,
    pub priority: Number,
    pub updated_at: String,
    pub memex_project_column_values: Vec<MemexProjectColumn>,
    pub content: MemexContent,
}

impl MemexItem {
    /// returns whether one of assignees matches
    pub fn contains_assignee(&self, username: &str) -> bool {
        let mut found = false;
        for col in &self.memex_project_column_values {
            match col {
                MemexProjectColumn::Assignees { assignees } => {
                    for a in assignees {
                        if a.login == username {
                            found = true;
                        }
                    }
                }
                _ => {}
            }
        }
        found
    }

    /// get status of the item
    pub fn status_id(&self) -> String {
        let mut status_id = "".to_string();
        for col in &self.memex_project_column_values {
            match col {
                MemexProjectColumn::Status { id } => {
                    status_id = id.to_string();
                }
                _ => {}
            }
        }
        status_id
    }

    /// number of the issue
    pub fn number(&self) -> u64 {
        let mut number = 0;
        for col in &self.memex_project_column_values {
            match col {
                MemexProjectColumn::Title { number: n, .. } => {
                    number = *n;
                }
                _ => {}
            }
        }
        number
    }

    /// title of the item
    pub fn title(&self) -> String {
        let mut title = "".to_string();
        for col in &self.memex_project_column_values {
            match col {
                MemexProjectColumn::Title { title: t, .. } => {
                    title = t.to_string();
                }
                _ => {}
            }
        }
        title
    }

    /// assignees list as a comma-separated string
    pub fn assignees(&self) -> String {
        let mut out = "".to_string();
        for col in &self.memex_project_column_values {
            match col {
                MemexProjectColumn::Assignees { assignees } => {
                    out = assignees
                        .iter()
                        .map(|a| a.login.clone())
                        .collect::<Vec<String>>()
                        .join(",");
                }
                _ => {}
            }
        }
        out
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexTitle {
    pub raw: String,
    pub html: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexAssignee {
    pub login: String,
    pub id: i64,
    // pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "memexProjectColumnId", content = "value")]
pub enum MemexProjectColumn {
    #[serde(rename = "Title", deserialize_with = "deserialize_title")]
    Title { title: String, number: u64 },
    #[serde(rename = "Status", deserialize_with = "deserialize_status")]
    Status { id: String },
    #[serde(rename = "Assignees", deserialize_with = "deserialize_assignees")]
    Assignees { assignees: Vec<MemexAssignee> },
    #[serde(rename = "Repository", deserialize_with = "deserialize_repo")]
    Repository { name: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemexContent {
    pub id: i64,
}

use serde::de::Deserializer;

fn deserialize_title<'de, D>(deserializer: D) -> Result<(String, u64), D::Error>
where
    D: Deserializer<'de>,
{
    let r: Value = Deserialize::deserialize(deserializer)?;
    if r.is_null() {
        Ok(("".to_string(), 0))
    } else if r.is_string() {
        // i.e "you can't see that item"
        Ok((r.as_str().unwrap().to_string(), 0))
    } else {
        let number = r["number"].as_u64().unwrap_or_default();
        let t = r["title"].clone();
        let title = t["raw"].as_str().unwrap_or_default().to_string();
        Ok((title, number))
    }
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let r: Value = Deserialize::deserialize(deserializer)?;
    if r.is_null() {
        Ok("".to_string())
    } else {
        let id = r["id"].as_str().unwrap_or_default().to_string();
        Ok(id)
    }
}

fn deserialize_repo<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let r: Value = Deserialize::deserialize(deserializer)?;
    let name = r["name"].as_str().unwrap_or_default().to_string();
    Ok(name)
}

fn deserialize_assignees<'de, D>(deserializer: D) -> Result<Vec<MemexAssignee>, D::Error>
where
    D: Deserializer<'de>,
{
    let a: Value = Deserialize::deserialize(deserializer)?;
    if a.is_null() {
        Ok(vec![])
    } else {
        let assignees: Vec<MemexAssignee> = serde_json::from_value(a).unwrap();
        Ok(assignees)
    }
}
