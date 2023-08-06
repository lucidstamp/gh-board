use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemexDataType {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub id: String,
    pub settings: Option<MemexDataTypeSettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemexDataTypeSettings {
    pub options: Option<Vec<MemexDataTypeOption>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemexDataTypeOption {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemexItem {
    // pub content_id: i64,
    // pub content_type: String,
    // pub content_repository_id: Value,
    // pub id: i64,
    // pub priority: Number,
    // pub updated_at: String,
    pub memex_project_column_values: Vec<MemexProjectColumn>,
    // pub content: MemexContent,
}

pub trait IMemexItem {
    fn contains_assignee(&self, username: &str) -> bool;
    fn status_id(&self) -> String;
    fn number(&self) -> u64;
    fn title(&self) -> String;
    fn assignees(&self) -> String;
}

impl IMemexItem for MemexItem {
    /// returns whether one of assignees matches
    fn contains_assignee(&self, username: &str) -> bool {
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
    fn status_id(&self) -> String {
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
    fn number(&self) -> u64 {
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
    fn title(&self) -> String {
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
    fn assignees(&self) -> String {
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemexAssignee {
    pub login: String,
    // pub id: i64,
    // pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "memexProjectColumnId", content = "value")]
enum MemexProjectColumn {
    #[serde(rename = "Title", deserialize_with = "deserialize_title")]
    Title { title: String, number: u64 },
    #[serde(rename = "Status", deserialize_with = "deserialize_status")]
    Status { id: String },
    #[serde(rename = "Assignees", deserialize_with = "deserialize_assignees")]
    Assignees { assignees: Vec<MemexAssignee> },
    #[serde(rename = "Repository", deserialize_with = "deserialize_repo")]
    Repository { _name: String },
}

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct MemexContent {
// id: i64,
// }

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

pub struct Board {
    items: Vec<MemexItem>,
    columns: Vec<MemexDataType>,
}

impl Board {
    /// returns the items on the board
    pub fn items(&self) -> Vec<impl IMemexItem> {
        self.items.clone()
    }

    /// returns the human readable name of a status from its ID
    pub fn get_status_name(&self, id: &str) -> Option<&str> {
        let status = self.columns.iter().find(|c| c.id == "Status")?;
        if let Some(settings) = status.settings.as_ref() {
            let options = settings.options.as_ref()?;
            let option = options.iter().find(|o| o.id == id)?;
            return Some(&option.name);
        }
        None
    }

    /// initialize from local files - for testing
    pub fn from_local_path(path: &str) -> Self {
        let path_items = format!("{}/memex-items-data.json", path);
        let data = std::fs::read_to_string(path_items).unwrap();
        let items: Vec<MemexItem> = serde_json::from_str(&data).unwrap();

        let path_columns = format!("{}/memex-columns-data.json", path);
        let data = std::fs::read_to_string(path_columns).unwrap();
        let columns: Vec<MemexDataType> = serde_json::from_str(&data).unwrap();

        Self { items, columns }
    }

    /// initialize from remote HTML page
    pub fn from_url(url: &str, user_session: &str, github_session: &str) -> Self {
        let cookie = format!("user_session={}; _gh_sess={}", user_session, github_session);
        let client = reqwest::blocking::Client::new()
            .get(url)
            .header("Cookie", cookie)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
        let data = client.send().unwrap().text().unwrap();
        println!("downloaded: {} bytes", data.len());
        let dom = tl::parse(&data, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let memex_items_data = dom
            .get_element_by_id("memex-items-data")
            .expect("Failed to find #memex-items-data")
            .get(parser)
            .unwrap()
            .inner_text(parser);
        let memex_columns_data = dom
            .get_element_by_id("memex-columns-data")
            .expect("Failed to find #memex-columns-data")
            .get(parser)
            .unwrap()
            .inner_text(parser);

        let items: Vec<MemexItem> = serde_json::from_str(&memex_items_data).unwrap();
        let columns: Vec<MemexDataType> = serde_json::from_str(&memex_columns_data).unwrap();
        Self { items, columns }
    }
}
