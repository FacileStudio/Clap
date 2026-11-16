# Motion starters

Ready-made scenes. Each file is one complete scene: a frontmatter block with `layout: raw` and
`duration`, then a self-contained HTML composition that animates only through
`scene.timeline`. Every id and class starts with `st-<name>`, so several starters can sit in
one project.

| File | Seconds | What it is |
|---|---|---|
| `title.md` | 4 | A large title, a subtitle and an accent bar that rise in and fade out. |
| `lower-third.md` | 5 | A name plate that slides in at the bottom left, holds, and leaves. |
| `caption.md` | 6 | Three captions shown one after the other as clips. |
| `chart-race.md` | 6 | Four bars that grow, then change length and rank. |
| `logo-reveal.md` | 3 | A mark that spins in, then the name. |

`clap starters` lists them and `clap new <name> --template <starter>` starts a project from one.
By hand: put a headmatter above the file, or use it as it is. To insert one
into a project, pass everything after its frontmatter as the `body` of an `add_scene` op with
the file's `duration`.

`cargo test -p clap-render --test starters` parses, compiles and renders each one.
