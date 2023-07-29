pub mod args;
pub mod proto;

use clap::Parser;
use color_eyre::owo_colors::OwoColorize;

pub struct Board {
    items: Vec<proto::MemexItem>,
    columns: Vec<proto::MemexDataType>,
}

impl Board {
    /// returns the human readable name of a status from its ID
    fn get_status_name(&self, id: &str) -> Option<&str> {
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
        let items: Vec<proto::MemexItem> = serde_json::from_str(&data).unwrap();

        let path_columns = format!("{}/memex-columns-data.json", path);
        let data = std::fs::read_to_string(path_columns).unwrap();
        let columns: Vec<proto::MemexDataType> = serde_json::from_str(&data).unwrap();

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

        let items: Vec<proto::MemexItem> = serde_json::from_str(&memex_items_data).unwrap();
        let columns: Vec<proto::MemexDataType> = serde_json::from_str(&memex_columns_data).unwrap();
        Self { items, columns }
    }
}

fn main() {
    let opt = args::Cli::parse();
    match opt.cmd {
        args::Command::List { status, user } => {
            // let board = Board::from_local_path("/home/adm0/src/temp");
            let board = Board::from_url(&opt.url, &opt.user_session, &opt.github_session);
            // split status into vec separated by comma
            let statuses: Vec<&str> = status.split(',').collect();
            for status_id in statuses {
                println!(
                    "{}",
                    board
                        .get_status_name(status_id)
                        .unwrap_or(status_id)
                        .yellow()
                );
                for item in board
                    .items
                    .iter()
                    // filter by board status
                    .filter(|i| i.status_id() == status_id)
                    // filter by user, if provided
                    .filter(|i| {
                        if let Some(u) = &user {
                            i.contains_assignee(u)
                        } else {
                            true
                        }
                    })
                {
                    println!(
                        "{}\t{} {}",
                        format!("#{}", item.number()).green(),
                        item.title().white(),
                        item.assignees().magenta(),
                    );
                }
            }
        }
    }
}
