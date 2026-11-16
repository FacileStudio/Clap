//! The command line as clap reads it.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// The whole command line.
#[derive(Parser, Debug)]
#[command(name = "clap", version, about = "Write, preview, edit, render and sync videos kept as one text file")]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
    /// The flags every command takes.
    #[command(flatten)]
    pub globals: Globals,
}

/// The flags every command takes.
#[derive(Args, Debug, Default, Clone)]
pub struct Globals {
    /// `--json`.
    #[arg(long, global = true, help = "Print one JSON document instead of text")]
    pub json: bool,
    /// `--no-color`.
    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,
    /// `--server`.
    #[arg(long, global = true, value_name = "url", help = "Use this Clap server")]
    pub server: Option<String>,
}

/// Every command, in two groups: the ones that work offline and the ones that talk to a server.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// The commands that need no server.
    #[command(flatten)]
    Local(Local),
    /// The commands that talk to a Clap server.
    #[command(flatten)]
    Remote(Remote),
}

/// The commands that need no server.
#[derive(Subcommand, Debug)]
pub enum Local {
    /// Create a folder with a starter project
    New(NewArgs),
    /// List the starters `new --template` takes
    Starters,
    /// Serve a project locally with a player, and reload it on every change
    Serve(ServeArgs),
    /// Validate a project and list every problem with its line
    Check(FolderArgs),
    /// Print the format reference
    Guide,
    /// Print the timeline: scenes, tracks and clips
    Timeline(FolderArgs),
    /// Apply timeline operations to the project file
    Edit(EditArgs),
    /// Render a project to an MP4, here or on the server
    Render(RenderArgs),
}

/// The commands that talk to a Clap server.
#[derive(Subcommand, Debug)]
pub enum Remote {
    /// Sign in to a Clap server
    Login(LoginArgs),
    /// Remove the stored login
    Logout,
    /// Show the signed-in account
    Whoami,
    /// List your projects
    List(ListArgs),
    /// Download a project and its assets into a folder
    Pull(PullArgs),
    /// Upload a folder's project and assets
    Push(PushArgs),
    /// Open a pulled project in the web app
    Open(OpenArgs),
}

/// `clap new`.
#[derive(Args, Debug)]
pub struct NewArgs {
    /// The folder to create.
    #[arg(value_name = "name", help = "Folder to create")]
    pub name: PathBuf,
    /// The starter the project begins with. Without it, the welcome scene in the language of
    /// `LC_ALL`, `LC_MESSAGES` or `LANG`.
    #[arg(
        short,
        long,
        value_name = "starter",
        help = "Starter to begin with, see `clap starters` [default: a welcome scene in your language]"
    )]
    pub template: Option<String>,
}

/// A command that takes only the project folder.
#[derive(Args, Debug)]
pub struct FolderArgs {
    /// The project folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Project folder or file [default: .]")]
    pub dir: Option<PathBuf>,
}

/// `clap serve`.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// The project folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Project folder or file [default: .]")]
    pub dir: Option<PathBuf>,
    /// The port to listen on.
    #[arg(short, long, default_value_t = 3030, value_name = "port", help = "Port to listen on, 0 for any free port")]
    pub port: u16,
}

/// `clap edit`.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// The project folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Project folder or file [default: .]")]
    pub dir: Option<PathBuf>,
    /// The operations: a JSON array, one JSON object, or `@file`.
    #[arg(
        long,
        value_name = "json|@file",
        help = "Operations as JSON, or @file to read them from a file, @- for stdin"
    )]
    pub ops: String,
}

/// `clap render`.
#[derive(Args, Debug)]
pub struct RenderArgs {
    /// The project folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Project folder or file [default: .]")]
    pub dir: Option<PathBuf>,
    /// The file to write.
    #[arg(short, long, value_name = "file", help = "File to write [default: <folder name>.mp4]")]
    pub output: Option<PathBuf>,
    /// The frame width.
    #[arg(long, value_name = "pixels", help = "Frame width, even [default: the project's size]")]
    pub width: Option<u32>,
    /// The frame height.
    #[arg(long, value_name = "pixels", help = "Frame height, even [default: the project's size]")]
    pub height: Option<u32>,
    /// The frame rate.
    #[arg(long, value_name = "fps", help = "Frames per second [default: 30]")]
    pub fps: Option<u32>,
    /// The seconds to render.
    #[arg(long, value_name = "a:b", help = "Seconds to render, as start:end [default: the whole project]")]
    pub range: Option<String>,
    /// Render on the server.
    #[arg(long, help = "Render the pushed project on the server and download the MP4")]
    pub remote: bool,
}

/// `clap login`.
#[derive(Args, Debug)]
pub struct LoginArgs {
    /// The server to sign in to.
    #[arg(
        value_name = "server",
        help = "Server to sign in to [default: the stored one, or https://clap.facile.studio]"
    )]
    pub target: Option<String>,
    /// Use the device grant even where a browser can open.
    #[arg(long, help = "Approve the login from another device instead of this machine's browser")]
    pub device: bool,
    /// Print the sign-in link and open nothing.
    #[arg(long, help = "Print the sign-in link without opening a browser")]
    pub no_browser: bool,
}

/// `clap list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Print bare project ids.
    #[arg(short, long, help = "Print project ids only")]
    pub quiet: bool,
}

/// `clap pull`.
#[derive(Args, Debug)]
pub struct PullArgs {
    /// A project id or its web URL. Without it, the folder's own project.
    #[arg(value_name = "project", help = "Project id or URL [default: the project this folder was pulled from]")]
    pub project: Option<String>,
    /// The folder to write into.
    #[arg(value_name = "dir", help = "Folder to write into [default: a folder named after the project, or .]")]
    pub dir: Option<PathBuf>,
    /// Discard local changes.
    #[arg(long, help = "Overwrite local changes")]
    pub force: bool,
}

/// `clap push`.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// The project folder.
    #[arg(value_name = "dir", help = "Project folder [default: .]")]
    pub dir: Option<PathBuf>,
    /// Overwrite what the server holds.
    #[arg(long, help = "Overwrite the server's copy even if it changed since the last pull")]
    pub force: bool,
    /// Create the project when the folder is not linked to one.
    #[arg(long, help = "Create a new project on the server from this folder")]
    pub create: bool,
}

/// `clap open`.
#[derive(Args, Debug)]
pub struct OpenArgs {
    /// The project folder.
    #[arg(value_name = "dir", help = "Project folder [default: .]")]
    pub dir: Option<PathBuf>,
    /// Print the URL and open nothing.
    #[arg(long, help = "Print the URL without opening a browser")]
    pub print: bool,
}
