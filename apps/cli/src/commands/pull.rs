//! `clap pull`: a project's source and assets into a folder, without losing local edits.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::api::{Client, Record};
use crate::cli::{Globals, PullArgs};
use crate::commands::{linked_session, print_json, remote_assets, session};
use crate::failure::{Failure, Outcome};
use crate::progress::Bar;
use crate::project::ENTRY;
use crate::sync::plan::{self, PullPlan, SourcePull};
use crate::sync::{Hashes, LINK_FILE, Link, local_assets, sha256};
use crate::{ui, url};

/// What a pull works on: the project as the server holds it, the folder, and the folder's link.
struct Wanted {
    client: Client,
    project: Record,
    dir: PathBuf,
    base: Link,
}

fn linked(dir: &Path, globals: &Globals) -> Outcome<Wanted> {
    let Some(base) = Link::load(dir)? else {
        let hint = "run `clap pull <id>`, `clap list` shows the ids";
        return Err(Failure::new("not_linked", format!("{} is not linked to a project — {hint}", dir.display())));
    };
    let client = linked_session(globals.server.as_deref(), &base)?;
    let project = client.project(&base.project)?;
    Ok(Wanted { client, project, dir: dir.to_owned(), base })
}

fn named(reference: &str, args: &PullArgs, globals: &Globals) -> Outcome<Wanted> {
    let (origin, id) = url::project_reference(reference);
    let client = session(globals.server.as_deref().or(origin.as_deref()))?;
    let project = client.project(&id)?;
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from(url::slug(&project.title, &project.id)));
    let fresh = Link { project: project.id.clone(), server: client.server().to_owned(), ..Link::default() };
    let base = Link::load(&dir)?.unwrap_or_else(|| fresh.clone());
    if (&base.project, &base.server) != (&fresh.project, &fresh.server) {
        let message = format!("{} is linked to another project — pull into another folder", dir.display());
        return Err(Failure::new("other_project", message));
    }
    Ok(Wanted { client, project, dir, base })
}

fn wanted(args: &PullArgs, globals: &Globals) -> Outcome<Wanted> {
    match &args.project {
        Some(folder) if args.dir.is_none() && Path::new(folder).join(LINK_FILE).is_file() => {
            linked(Path::new(folder), globals)
        }
        Some(reference) => named(reference, args, globals),
        None => linked(Path::new("."), globals),
    }
}

fn decide(wanted: &Wanted, remote: &Hashes, force: bool) -> Outcome<(SourcePull, PullPlan)> {
    let here = fs::read(wanted.dir.join(ENTRY)).ok().map(|bytes| sha256(&bytes));
    let source =
        plan::pull_source(&wanted.base.source_sha256, here.as_deref(), &sha256(wanted.project.source.as_bytes()));
    let assets = plan::pull(&wanted.base.assets, &local_assets(&wanted.dir, &wanted.base.assets)?, remote);
    let changed: Vec<&str> = (source == SourcePull::LocalChanges)
        .then_some(ENTRY)
        .into_iter()
        .chain(assets.conflicts.iter().map(String::as_str))
        .collect();
    if force || changed.is_empty() {
        return Ok((source, assets));
    }
    let hint = "run `clap push` to send them, or `clap pull --force` to discard them";
    Err(Failure::new("local_changes", format!("{} changed here since the last sync — {hint}", changed.join(", "))))
}

fn fetch(wanted: &Wanted, name: &str) -> Outcome {
    let path = wanted.dir.join(name);
    fs::create_dir_all(path.parent().unwrap_or(&wanted.dir))?;
    let partial = path.with_file_name(format!(".{}.part", path.file_name().unwrap_or_default().to_string_lossy()));
    let mut bar = Bar::new(format!("Downloading {name}"));
    let fetched = fs::File::create(&partial).map_err(Failure::from).and_then(|mut file| {
        let progress = &mut |done, total| bar.bytes(done, total);
        Ok(wanted.client.download_asset(&wanted.project.id, name, &mut file, progress)?)
    });
    bar.finish();
    let moved = fetched.and_then(|_| Ok(fs::rename(&partial, &path)?));
    if moved.is_err() {
        let _ = fs::remove_file(&partial);
    }
    moved
}

fn apply(wanted: &Wanted, remote: &Hashes, source: SourcePull, assets: &PullPlan) -> Outcome<Link> {
    let (dir, project) = (&wanted.dir, &wanted.project);
    fs::create_dir_all(dir)?;
    if source != SourcePull::UpToDate {
        fs::write(dir.join(ENTRY), &project.source)?;
    }
    for name in &assets.download {
        fetch(wanted, name)?;
    }
    assets.remove.iter().for_each(|name| drop(fs::remove_file(dir.join(name))));
    let held = local_assets(dir, remote)?.into_iter().filter(|(name, _)| remote.contains_key(name)).collect();
    let source_sha256 = sha256(project.source.as_bytes());
    let link = Link { rev: project.rev, source_sha256, assets: held, ..wanted.base.clone() };
    link.save(dir)?;
    Ok(link)
}

fn report(wanted: &Wanted, link: &Link, source: SourcePull, assets: &PullPlan) {
    let (dir, updated) = (wanted.dir.display().to_string(), source != SourcePull::UpToDate);
    if ui::json() {
        let assets = json!({ "downloaded": assets.download, "removed": assets.remove });
        let project = json!({ "project": link.project, "title": wanted.project.title, "rev": link.rev });
        return print_json(&json!({ "dir": dir, "source_updated": updated, "assets": assets, "project": project }));
    }
    if !ui::json() {
        assets.download.iter().for_each(|name| println!("\u{2193} {name}"));
        assets.remove.iter().for_each(|name| println!("- {name}"));
    }
    match updated || !assets.download.is_empty() || !assets.remove.is_empty() {
        true => ui::success(&format!("Pulled project {} into {dir} (revision {})", link.project, link.rev)),
        false => ui::success(&format!("{dir} is up to date (revision {})", link.rev)),
    }
}

/// Fetches the project, refuses to overwrite local edits without `--force`, then writes the
/// source, the changed assets and the link.
pub fn run(args: &PullArgs, globals: &Globals) -> Outcome {
    let wanted = wanted(args, globals)?;
    let remote = remote_assets(&wanted.client, &wanted.project.id)?;
    let (source, assets) = decide(&wanted, &remote, args.force)?;
    let link = apply(&wanted, &remote, source, &assets)?;
    report(&wanted, &link, source, &assets);
    Ok(())
}
