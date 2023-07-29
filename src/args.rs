use clap::{Parser, Subcommand};

#[derive(Clone, Subcommand)]
pub enum Command {
    /// display issues
    List {
        /// statuses to be displayed
        #[clap(short, long, env = "GH_BOARD_STATUSES")]
        status: String,
        /// filter results by assignee
        #[clap(short, long, env = "GH_BOARD_USER")]
        user: Option<String>,
    },
}

#[derive(Clone, Parser)]
#[clap(name = "gh-board", version = "1.0")]
pub struct Cli {
    /// URL of the project board,
    /// e.g. https://github.com/orgs/COMPANY/projects/PROJECT
    #[clap(short, long, env = "GH_BOARD_URL")]
    pub url: String,
    /// User session ID, should be taken from cookies
    #[clap(long, env = "GH_BOARD_USER_SESSION")]
    pub user_session: String,
    /// Github session ID, should be taken from cookies
    #[clap(long, env = "GH_BOARD_GITHUB_SESSION")]
    pub github_session: String,
    #[clap(subcommand)]
    pub cmd: Command,
}
