use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap as Map;

#[derive(Debug)]
pub struct Contributions {
    days: Map<String, u32>,
}

impl Contributions {
    /// pick latest days from contributions
    pub fn latest(&self, days: u32) -> Map<String, u32> {
        let mut latest = Map::new();
        let min_index: u32 = if self.days.len() as u32 > days {
            self.days.len() as u32 - days
        } else {
            0
        };
        let mut index = 0;
        for (day, num) in &self.days {
            if index >= min_index {
                latest.insert(day.clone(), *num);
            }
            index += 1;
        }
        latest
    }

    /// parse contributions from local HTML file
    pub fn from_local_file(path: &str) -> Self {
        let data = std::fs::read_to_string(path).unwrap();
        Self::from_str(&data)
    }

    /// parse from remote URL
    pub fn from_url(url: &str) -> Self {
        let data = reqwest::blocking::get(url).unwrap().text().unwrap();
        Self::from_str(&data)
    }

    /// parse from GitHub user
    pub fn from_github(user: &str) -> Self {
        let url = format!("https://github.com/{}", user);
        Self::from_url(&url)
    }

    /// parse contributions from HTML string
    pub fn from_str(contents: &str) -> Self {
        let dom = tl::parse(&contents, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let cells = dom
            .get_elements_by_class_name("ContributionCalendar-day")
            .into_iter();
        let mut days = Map::new();
        for c in cells {
            let p = c.get(parser).expect("parser failure");
            let attr = p.as_tag().expect("not a tag").attributes();
            let date = match attr.get("data-date") {
                Some(Some(date)) => date.as_utf8_str().to_string(),
                _ => "".to_string(),
            };
            if date.is_empty() {
                continue;
            }
            // parse date as chrono::NaiveDate and get weekday
            let d = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let date_str = format!("{}, {}", date, d.weekday());

            let span = p.children().expect("no children").all(parser)[0]
                .as_tag()
                .unwrap()
                .inner_text(parser);
            // collect digits from the beginning of the string
            let digits = span
                .chars()
                .take_while(|c| c.is_digit(10))
                .collect::<String>();
            let num = if digits.is_empty() {
                0
            } else {
                digits.parse::<u32>().unwrap()
            };
            days.insert(date_str, num);
        }
        Self { days }
    }
}
