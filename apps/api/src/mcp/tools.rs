use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::{Context, assets, delete, guide, read, render, timeline, write};

const LIST: &str = "The projects you reach, newest first, 50 at a time by default: your personal projects and \
those of your spaces, or one space with space_id. Call it to find a project id; pass next_cursor as cursor for \
the next window.";
const GET: &str = "One project: its whole source in the scene format, its rev, its scene count, its length in \
seconds and its asset names. Call it before any write: every write tool needs the rev it answers.";
const CREATE: &str = "A new project from a source in the scene format, at rev 1. Call it once per project: each \
call adds another project. An invalid source is refused with the line of each problem.";
const WRITE: &str = "Replaces the whole source of a project. Call it to rewrite a project; it needs the rev \
clap_get_project answered and is refused, changing nothing, when the project moved on or the source is \
invalid. The old source stays in the project's revisions.";
const EDIT: &str = "Replaces one exact string in a project's source. Call it for a small change. old_string \
must be in the source exactly once: zero or several matches are refused, so add surrounding text until it \
names one place. It needs the rev clap_get_project answered.";
const TIMELINE: &str = "Moves, trims, splits, deletes and adds clips, and sets, reorders, deletes and adds \
scenes, by clip id and scene index (from 0). ops is a list applied in order, each tagged by op: move_clip {clip, \
start, track?}, trim_clip {clip, start?, duration?, media_in?}, split_clip {clip, at}, delete_clip {clip}, \
add_clip {scene, kind, src?, start, duration, track, text?}, set_scene_duration {scene, duration}, \
reorder_scene {scene, to}, delete_scene {scene}, add_scene {at, duration, body}. Times are seconds. A clip id \
is the element's data-clip, else its id, else clip-<scene>-<n>: such a derived id shifts when a clip before it \
is added or deleted, so read the project again before the next call. It needs the rev clap_get_project \
answered; nothing is saved unless every op applies. It answers the new rev and the ids of the clips it changed.";
const ASSETS: &str = "The media of a project: each file's name, kind, length, size in pixels and state. Call it \
before you reference a file: a source may only name the files listed here, by their name.";
const GUIDE: &str = "The format reference: frontmatter keys, layouts, timelines and the parser's errors, then \
Clap's own rules for time-based scenes, clips, tracks, media and determinism. Call it once before you write or \
edit a project.";
const RENDER: &str = "Queues an MP4 render of the project as it is saved now, and answers the render id. \
Rendering takes minutes: follow it with clap_render_status. The size defaults to the project's own, the rate \
to 30 frames a second, the range to the whole project.";
const STATUS: &str = "Where a render is: queued, running with its frame count, done with the download link, \
failed with the reason, or cancelled. Call it again later while it is queued or running; do not loop tightly.";
const DELETE: &str = "Moves a project to the trash, where it stays 30 days before it is deleted for good. The \
person confirms first and nothing changes until they do.";

fn spec(name: &str, title: &str, description: &str, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: description.into(), annotations: Some(annotations) }
}

fn reads(ctx: &Context) -> Vec<Tool> {
    let (a, b, c, d) = (ctx.clone(), ctx.clone(), ctx.clone(), ctx.clone());
    vec![
        tool(spec("clap_list_projects", "List projects", LIST, Annotations::READ), move |call, input| {
            read::list(a.clone(), call, input)
        }),
        tool(spec("clap_get_project", "Read a project", GET, Annotations::READ), move |call, input| {
            read::get(b.clone(), call, input)
        }),
        tool(spec("clap_list_assets", "List the media of a project", ASSETS, Annotations::READ), move |call, input| {
            assets::list(c.clone(), call, input)
        }),
        tool(spec("clap_render_status", "Follow a render", STATUS, Annotations::READ), move |call, input| {
            render::status(d.clone(), call, input)
        }),
        tool(spec("clap_format_guide", "Read the format guide", GUIDE, Annotations::READ), guide::guide),
    ]
}

fn sources(ctx: &Context) -> Vec<Tool> {
    let (a, b, c) = (ctx.clone(), ctx.clone(), ctx.clone());
    vec![
        tool(spec("clap_create_project", "Create a project", CREATE, Annotations::ADD), move |call, input| {
            write::create(a.clone(), call, input)
        }),
        tool(spec("clap_write_project", "Rewrite a project", WRITE, Annotations::OVERWRITE), move |call, input| {
            write::write(b.clone(), call, input)
        }),
        tool(spec("clap_edit_project", "Edit a project", EDIT, Annotations::OVERWRITE), move |call, input| {
            write::edit(c.clone(), call, input)
        }),
    ]
}

fn actions(ctx: &Context) -> Vec<Tool> {
    let (a, b) = (ctx.clone(), ctx.clone());
    vec![
        tool(spec("clap_timeline_edit", "Edit the timeline", TIMELINE, Annotations::OVERWRITE), move |call, input| {
            timeline::edit(a.clone(), call, input)
        }),
        tool(spec("clap_render", "Render a project", RENDER, Annotations::ADD), move |call, input| {
            render::render(b.clone(), call, input)
        }),
    ]
}

/// The eleven tools.
pub fn all(ctx: &Context, confirmations: &Confirmations) -> Vec<Tool> {
    let (context, confirm) = (ctx.clone(), confirmations.clone());
    let mut tools = reads(ctx);
    tools.extend(sources(ctx));
    tools.extend(actions(ctx));
    tools.push(tool(
        spec("clap_delete_project", "Delete a project", DELETE, Annotations::REMOVE),
        move |call, input| delete::delete(context.clone(), confirm.clone(), call, input),
    ));
    tools
}
