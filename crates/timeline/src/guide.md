# Clap: a video is a scene file

Everything above applies to Clap. A Clap project is the same file, read as a video: scenes
play one after the other, each for its `duration`, and the project lasts the sum of its scenes.
This section says how to write one that previews and renders well.

## Write time-based scenes

Use `layout: raw` and set `duration` on every scene. A raw scene is a full-stage HTML
composition on a black stage of the size given by `size`. Place clips with the classes of "The
stage" below, or position them yourself, absolutely.

```md
---
title: Launch
size: 1920x1080
layout: raw
duration: 6
---

<video id="shot" class="clap-fill" src="assets/shot.mp4" data-start="0" data-duration="6" data-track-index="0" data-in="1.5"></video>
<h1 id="title" class="clap-title" data-start="1" data-duration="4" data-track-index="1">Launch day</h1>
<audio id="music" src="assets/music.mp3" data-start="0" data-duration="6" data-volume="0.4"></audio>
<script>
  scene.timeline((tl) => tl.from("#title", { opacity: 0, y: 40, duration: 0.8 }, 1));
</script>
```

Markdown scenes (`default`, `cover`, `center`, `split`) work too: a scene with steps lasts
`step_duration` per stop, which is how a deck becomes a video. Prefer raw scenes for motion.

## The stage

Clap adds a small stylesheet to every preview and every render. In a raw scene:

- The scene is a black stage with white text: the theme's `--bg`, `--fg`, `--muted`, `--fill`
  and `--fill-strong` are redefined for it. Set them on `.scene` to change them.
- Every element with `data-track-index` is positioned, so the track stacks it: a higher track
  draws on top, with no CSS from you.

Five classes place a clip. Each is one class selector: any rule you write wins over it.

| Class | Does | Use it on |
|---|---|---|
| `clap-fill` | Fills the whole stage, cropped to fit (`object-fit: cover`) | a full-frame `<video>` or `<img>` |
| `clap-layer` | A full-stage layer you lay out inside | an HTML clip |
| `clap-title` | Centered white title, 112 px, with a soft shadow that reads on video | a title |
| `clap-caption` | One or two centered lines near the bottom, 44 px | a subtitle or a hint |
| `clap-lower` | A name plate at the bottom left on a dark rounded panel | a lower third |

`add_clip` writes these classes: a `video` or `image` clip gets `clap-fill`, a `text` clip
`clap-title`, an `html` clip `clap-layer`. An `audio` clip draws nothing. To make a text clip a
lower third or a caption, change its class.

## Place a clip on the stage

Five attributes give a clip its box, in pixels of the frame (`size`), from the top left corner:
`data-x`, `data-y`, `data-w`, `data-h`, and `data-rot`, degrees clockwise around the box's
centre. Write all five together. A clip that carries none keeps the box its classes give it.

- `data-fit` says how a video or a picture fills its box: `cover` (crop, the default), `contain`
  (letterbox) or `fill` (stretch).
- `data-opacity` is from `0` to `1`.
- The box wins over the clip's class and over your CSS for position and size, so move a clip
  with its attributes, not with a stylesheet. Animate what is inside it with `scene.timeline`.

`add_clip` writes the full frame on a video or a picture: `data-x="0" data-y="0"`, the stage
width and height, `data-rot="0"` and `data-fit="cover"`. `update_clip` moves and resizes it.

```html
<img id="logo" src="assets/logo.png" alt="" data-start="1" data-duration="3" data-track-index="2"
     data-x="1560" data-y="60" data-w="300" data-h="120" data-rot="0" data-fit="contain">
```

## Transitions

A clip comes in with `data-enter` and goes out with `data-exit`: `fade`, `slide-left`,
`slide-right`, `slide-up`, `slide-down`, `zoom-in`, `zoom-out`, `wipe-left`, `wipe-right`,
`blur` or `pop`. Each lasts `data-enter-duration` or `data-exit-duration` seconds, 0.5 without
it, and at most half the clip. A sound or a video with `fade` also fades its sound.

A scene comes in with the frontmatter key `transition`: `fade` (a crossfade with the scene
before), `dip` (through black: the scene before fades out, then this one fades in), `slide-left`
and `slide-right` (this scene pushes the one before out), `wipe-left`, `wipe-right` or `zoom`.
`transition_duration` sets its length in seconds, 0.6 without it. The scene before holds its
last frame while the next one comes in, so no time is added.

```md
---
layout: raw
duration: 5
transition: fade
transition_duration: 0.8
---
```

Every transition is a function of the time alone, eased in and out, so the preview and the
render show the same frame at the same time. Do not add CSS transitions for these.

## Clips

A clip is any element with `data-start`. It exists from `data-start` for `data-duration`
seconds, counted from the start of its scene. The timeline in the editor shows one bar per clip.

- Give every clip an `id`, unique in the file. The timeline tools name a clip by `data-clip`,
  else by its `id`. A clip with neither is named `clip-<scene>-<n>`, a derived id that shifts
  when a clip before it is added or deleted: read the project again after such an edit.
- Keep one clip per line when you can: an edit then changes one line.
- A clip must end inside its scene: `data-start + data-duration <= duration`.
- Without `data-duration` a clip lasts to the end of its scene.
- A clip may hold other elements. Animate what is inside it with `scene.timeline`.

## Tracks

`data-track-index` is the clip's track, a whole number. A higher track draws on top: it becomes
the element's `z-index`. Two clips of one scene that share a track must not overlap in time:
`clap_timeline_edit` refuses an edit that makes them overlap.
Put the background video on track 0, pictures above it, titles on top. Sound needs no track.

## Media

Media are the project's assets. `clap_list_assets` lists them with their `name`, `kind`,
`duration_ms`, size in pixels and `state`. Reference an asset by its `name`, exactly, as a
relative path: `src="assets/shot.mp4"`. A remote URL is refused, and so is a file the project
does not hold when you render. Assets are uploaded by people in the editor: you cannot upload
one, so use only the names the list answers.

- `<video src data-start data-duration>` plays a video for the life of the clip.
- `<audio src data-start data-duration>` plays a sound. It draws nothing.
- `<img src>` shows a picture. Add `data-start` and `data-duration` to make it a clip.
- `data-in` is the second of the media file the clip starts at. With `data-in="1.5"` a clip
  shows the file from 1.5 s on. Without it the file plays from its start.
- `data-volume` is the loudness from `0` to `1`. Without it the media plays at full volume.
- Never write `autoplay`, `loop`, `controls` or `muted`. The clip decides when media plays.
- A clip longer than what is left of its file holds the last frame. Check `duration_ms`.
- An asset whose `state` is `probing` is usable; `failed` means the file could not be read.

## Determinism

The renderer never plays the page. For each frame it calls `window.__scene.seek(t)` and
captures, so the frame at time `t` must depend on `t` alone.

- Animate only with `scene.timeline`. CSS transitions, CSS animations and
  `requestAnimationFrame` loops follow the wall clock and are not captured reliably.
- No `Date`, no `performance.now()`, no timers, no unseeded `Math.random()`, no network.
- Do not set `currentTime` on media yourself and do not call `play()`: write `data-in`.
- Fonts: use the theme's system stack, or a font file uploaded to the project and loaded with
  `@font-face { src: url(assets/font.woff2) }`. A remote font is refused.
- The preview plays a 720p proxy and is not frame-exact. The render is the reference.

## Editing with tools

1. `clap_get_project` answers the source and its `rev`. Every write names that `rev`.
2. `clap_edit_project` replaces one exact string: the right tool to change a text, a time or a
   style. `clap_write_project` replaces the whole file.
3. `clap_timeline_edit` applies timeline operations by clip id or scene index (scenes count
   from 0): `move_clip {clip, start, track?}`, `trim_clip {clip, start?, duration?, media_in?}`,
   `split_clip {clip, at}`, `delete_clip {clip}`,
   `add_clip {scene, kind, src?, start, duration, track, text?}` with `kind` one of `video`,
   `audio`, `image`, `text`, `html`, `set_scene_duration {scene, duration}`,
   `reorder_scene {scene, to}`, `delete_scene {scene}`, `add_scene {at, duration, body}`,
   `update_clip {clip, x?, y?, w?, h?, rot?, fit?, opacity?, volume?, enter?, exit?,
   enter_duration?, exit_duration?}`, where a `null` removes the attribute, and
   `set_scene_transition {scene, transition, duration?}`, where a `null` transition is a cut.
   It changes only the attribute values and the lines an operation concerns, and answers the
   new `rev` and the ids of the clips it changed. `split_clip` names the second half `<id>-b`.
4. `clap_render` queues an MP4 of the saved project and `clap_render_status` follows it. A
   render is at most 1920x1080, 30 frames a second and 10 minutes unless the instance says
   otherwise.
