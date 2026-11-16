//! `clap push`: the folder's source over the revision it was pulled at, then its assets.

use scene::Project;
use serde_json::json;

use crate::api::{ApiError, Client, Upload};
use crate::cli::{Globals, PushArgs};
use crate::commands::{linked_session, print_json, remote_assets, session};
use crate::failure::{Failure, Outcome};
use crate::login::loopback::nonce;
use crate::progress::Bar;
use crate::project::Folder;
use crate::sync::plan::{self, PushPlan};
use crate::sync::{Link, local_assets, sha256};
use crate::{diagnose, mime, ui};

const NOT_LINKED: &str = "run `clap push --create` to make one, or `clap pull <id>` to fetch one";

fn refused(error: ApiError, folder: &Folder, rev: i64) -> Failure {
    match error {
        ApiError::Invalid(errors) => {
            let label = folder.label();
            errors.iter().for_each(|error| eprintln!("{label}:{}: {}", error.line, error.message));
            Failure::new("invalid", format!("the server refused {label}: it has {}", diagnose::count(errors.len())))
        }
        error if error.is_conflict() => Failure::new(
            "conflict",
            format!(
                "the project changed on the server since revision {rev} — run `clap pull` to fetch it, \
                 or `clap push --force` to overwrite it"
            ),
        ),
        other => other.into(),
    }
}

fn create(globals: &Globals, folder: &Folder, source: &str, project: &Project) -> Outcome<(Client, Link)> {
    let client = session(globals.server.as_deref())?;
    let title = Some(project.config().title.clone()).filter(|title| !title.is_empty()).unwrap_or_else(|| folder.name());
    let project = client.create_project(&title, source).map_err(|error| refused(error, folder, 0))?;
    ui::step(&format!("Created project {}", project.id));
    let link = Link {
        project: project.id,
        server: client.server().to_owned(),
        rev: project.rev.unwrap_or(1),
        ..Link::default()
    };
    Ok((client, link))
}

fn write(args: &PushArgs, globals: &Globals, folder: &Folder, source: &str) -> Outcome<(Client, Link)> {
    let link = Link::load(&folder.dir)?.ok_or_else(|| {
        Failure::new("not_linked", format!("{} is not linked to a project — {NOT_LINKED}", folder.dir.display()))
    })?;
    let client = linked_session(globals.server.as_deref(), &link)?;
    let rev = if args.force { client.project(&link.project)?.rev } else { link.rev };
    let rev = client.put_source(&link.project, source, rev).map_err(|error| refused(error, folder, rev))?;
    Ok((client, Link { rev, ..link }))
}

fn too_large(name: &str, message: &str) -> Failure {
    let caps = "by default the server takes 2 GiB per file and 20 GiB for all the projects of one space";
    Failure::new("asset_too_large", format!("the server refused {name}: {message} — {caps}"))
}

fn upload(client: &Client, folder: &Folder, project: &str, name: &str) -> Outcome {
    let path = folder.dir.join(name);
    let kind = mime::of(&path).split(';').next().unwrap_or_default();
    let boundary = format!("clap-{}", nonce()?);
    let mut bar = Bar::new(format!("Uploading {name}"));
    let upload = Upload { name, kind, path: &path, boundary: &boundary };
    let sent = client.upload_asset(project, &upload, &mut |done, total| bar.bytes(done, total));
    bar.finish();
    sent.map_err(|error| match error {
        ApiError::Refused { status: 400, message, .. } => Failure::new(
            "asset_refused",
            format!("the server refused {name}: {message} — a project takes video, audio, images and fonts, named with letters, digits, `.`, `_` and `-`"),
        ),
        ApiError::Refused { status: 413, message, .. } => too_large(name, &message),
        other => other.into(),
    })
}

fn assets(client: &Client, folder: &Folder, link: &mut Link) -> Outcome<PushPlan> {
    let local = local_assets(&folder.dir, &link.assets)?;
    let plan = plan::push(&link.assets, &local, &remote_assets(client, &link.project)?);
    for name in &plan.upload {
        ui::step(&format!("Uploading {name}"));
        upload(client, folder, &link.project, name)?;
    }
    for name in &plan.delete {
        ui::step(&format!("Deleting {name} on the server"));
        client.delete_asset(&link.project, name)?;
    }
    link.assets = local;
    Ok(plan)
}

fn report(folder: &Folder, link: &Link, plan: &PushPlan) {
    if ui::json() {
        let (uploaded, deleted) = (&plan.upload, &plan.delete);
        let assets = json!({ "uploaded": uploaded, "deleted": deleted, "remote_only": plan.remote_only });
        return print_json(&json!({ "project": link.project, "rev": link.rev, "assets": assets }));
    }
    ui::success(&format!("Pushed {} to project {} (revision {})", folder.label(), link.project, link.rev));
    if !plan.remote_only.is_empty() {
        ui::hint(&format!("{} only on the server, run `clap pull` to fetch", plan.remote_only.join(", ")));
    }
}

/// Validates the project, writes its source over the linked revision, or creates the project with
/// `--create`, then syncs the assets. A stale revision stops the push before anything is
/// written.
pub fn run(args: &PushArgs, globals: &Globals) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let (source, project) = diagnose::require(&folder)?;
    let linked = Link::load(&folder.dir)?.is_some();
    let (client, mut link) = match args.create && !linked {
        true => create(globals, &folder, &source, &project)?,
        false => write(args, globals, &folder, &source)?,
    };
    link.source_sha256 = sha256(source.as_bytes());
    link.save(&folder.dir)?;
    let plan = assets(&client, &folder, &mut link)?;
    link.save(&folder.dir)?;
    report(&folder, &link, &plan);
    Ok(())
}
