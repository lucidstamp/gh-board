use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap as Map;

fn human_date(month: u32, day: u32) -> Result<String, &'static str> {
    let month_full_str = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June", // 6
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October", // 10
        11 => "November",
        12 => "December",
        _ => return Err("invalid month"),
    };
    let day_str = match day {
        1 => "1st".to_owned(),
        2 => "2nd".to_owned(),
        3 => "3rd".to_owned(),
        4..=20 => format!("{}th", day),
        21 => "21st".to_owned(),
        22 => "22nd".to_owned(),
        23 => "23rd".to_owned(),
        24..=30 => format!("{}th", day),
        31 => "31st".to_owned(),
        _ => return Err("invalid day"),
    };
    Ok(format!("{} {}", month_full_str, day_str))
}

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
        if contents.is_empty() {
            panic!("github user page is empty")
        }
        let dom = tl::parse(&contents, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut days = Map::new();

        let tooltips = dom
            .get_elements_by_class_name("sr-only")
            .into_iter()
            .filter(|x| {
                let node = x.get(parser).unwrap();
                let name = node.as_tag().unwrap().name().as_utf8_str();
                let dirty_inner = node.as_tag().unwrap().inner_text(parser);
                name == "tool-tip" && dirty_inner.contains("contribution")
            })
            .map(|x| x.get(parser).unwrap())
            .collect::<Vec<_>>();

        let mut human_dates: Map<String, u32> = Map::new();
        for t in tooltips {
            let dirty_inner = t
                .as_tag()
                .unwrap()
                .inner_text(parser)
                .replace("\n", " ")
                .replace(".", "");
            // clean replace multiple spaces with single space
            let inner = dirty_inner.split_whitespace().collect::<Vec<_>>().join(" ");

            if inner.contains("1 contribution ") {
                let parts = inner.split(" contribution on ").collect::<Vec<_>>();
                human_dates.insert(parts[1].to_string(), 1);
                continue;
            }

            let parts = inner.split(" contributions on ").collect::<Vec<_>>();
            let amt = if parts[0] == "No" {
                0
            } else {
                parts[0].parse::<u32>().unwrap()
            };
            human_dates.insert(parts[1].to_string(), amt);
        }
        // for (k, v) in &human_dates { println!("{} -> {}", k, v); }

        let cells = dom
            .get_elements_by_class_name("ContributionCalendar-day")
            .into_iter();
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

            let d = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            let date_str = format!("{}, {}", date, d.weekday());
            let month = d.month();
            let day = d.day();
            match human_date(month, day) {
                Ok(dt) => match human_dates.get(&dt) {
                    Some(num) => {
                        days.insert(date_str, *num);
                    }
                    None => {
                        println!("ERROR: {} -> {dt:?} NO MATCH IN TOOLTIPS ", date_str);
                    }
                },
                Err(e) => {
                    println!(
                        "ERROR: {} FAILURE IN HUMAN DATE {}",
                        date_str,
                        e.to_string()
                    );
                }
            }
        }
        Self { days }
    }
}
