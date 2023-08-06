use clap::{Parser, Subcommand};

#[derive(Clone, Subcommand)]
pub enum Command {
    /// display issues from Github Board
    List {
        /// URL of the project board,
        /// e.g. https://github.com/orgs/COMPANY/projects/PROJECT
        #[clap(long, env = "GH_BOARD_URL")]
        url: String,
        /// User session ID, should be taken from cookies
        #[clap(long, env = "GH_BOARD_USER_SESSION")]
        user_session: String,
        /// Github session ID, should be taken from cookies
        #[clap(long, env = "GH_BOARD_GITHUB_SESSION")]
        github_session: String,
        /// statuses to be displayed
        #[clap(short, long, env = "GH_BOARD_STATUSES")]
        status: String,
        /// filter results by assignee
        #[clap(short, long, env = "GH_BOARD_USER")]
        user: Option<String>,
    },
    /// display user Github user contributions
    #[clap(name = "contrib")]
    Contributions {
        /// filter results by assignee
        #[clap(short, long, default_value = "", env = "GH_BOARD_USER")]
        user: String,
        /// number of days to display
        #[clap(short, long, default_value = "28")]
        days: u32,
    },
}

#[derive(Clone, Parser)]
#[clap(name = "gh-board", version = "1.0")]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: Command,
}
