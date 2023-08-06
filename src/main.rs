pub mod args;
pub mod board;
pub mod contrib;

use board::{Board, IMemexItem};
use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use contrib::Contributions;

fn main() {
    let opt = args::Cli::parse();
    match opt.cmd {
        args::Command::Contributions { user, days } => {
            let gh_user = if user.is_empty() {
                // return user from running `git config --global user.name`
                let output = std::process::Command::new("git")
                    .args(&["config", "--global", "user.name"])
                    .output()
                    .expect("failed to execute git config");
                String::from_utf8(output.stdout).unwrap().trim().to_string()
            } else {
                user
            };
            if gh_user.is_empty() {
                panic!("No user provided and no git user found");
            }
            println!("{}, last {} days:", gh_user.yellow(), days.green(),);
            // let contrib = Contributions::from_local_file("/home/adm0/src/temp/contributions.html");
            let contrib = Contributions::from_github(&gh_user);
            for (day, num) in &contrib.latest(days) {
                let weekend = day.contains(", Sat") || day.contains(", Sun");
                if weekend {
                    print!("{}", day.magenta().strikethrough());
                    println!(": {}", num.magenta());
                } else {
                    print!("{}", day);
                    if *num > 3 {
                        println!(": {}", num.yellow());
                    } else if *num > 0 {
                        println!(": {}", num.green());
                    } else {
                        println!(": {}", num.red());
                    }
                };
            }
        }

        args::Command::List {
            status,
            user,
            url,
            user_session,
            github_session,
        } => {
            // let board = Board::from_local_path("/home/adm0/src/temp");
            let board = Board::from_url(&url, &user_session, &github_session);
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
                    .items()
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
