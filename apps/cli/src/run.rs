//! From a parsed command line to an exit code.

use std::process::ExitCode;
use std::sync::atomic::Ordering;

use crate::cli::{Cli, Command, Globals, Local, Remote};
use crate::failure::{INTERRUPTED, Outcome};
use crate::{commands, serve, ui};

fn local(command: &Local, globals: &Globals) -> Outcome {
    match command {
        Local::New(args) => commands::new::run(args),
        Local::Starters => commands::starters::run(),
        Local::Serve(args) => serve::run(args),
        Local::Check(args) => commands::check::run(args),
        Local::Guide => commands::guide::run(),
        Local::Timeline(args) => commands::timeline::run(args),
        Local::Edit(args) => commands::edit::run(args),
        Local::Render(args) if args.remote => commands::remote_render::run(args, globals),
        Local::Render(args) => commands::render::run(args),
    }
}

fn remote(command: &Remote, globals: &Globals) -> Outcome {
    match command {
        Remote::Login(args) => commands::login::run(args, globals),
        Remote::Logout => commands::logout::run(),
        Remote::Whoami => commands::whoami::run(globals),
        Remote::List(args) => commands::list::run(args, globals),
        Remote::Pull(args) => commands::pull::run(args, globals),
        Remote::Push(args) => commands::push::run(args, globals),
        Remote::Open(args) => commands::open::run(args),
    }
}

/// Runs the command and answers the exit code: 0, 1 after printing why, or 130 when Ctrl+C
/// stopped it.
pub fn run(cli: Cli) -> ExitCode {
    ui::JSON.store(cli.globals.json, Ordering::Relaxed);
    ui::PLAIN.store(cli.globals.no_color, Ordering::Relaxed);
    let outcome = match &cli.command {
        Command::Local(command) => local(command, &cli.globals),
        Command::Remote(command) => remote(command, &cli.globals),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            failure.report();
            ExitCode::from(if failure.code == INTERRUPTED { 130 } else { 1 })
        }
    }
}
