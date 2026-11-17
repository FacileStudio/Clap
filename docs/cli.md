# Clap: command line

`clap` writes, previews, edits, renders and syncs video projects from a terminal. It is the
`clap-cli` package in `apps/cli`. The local commands need no server.

It is the sibling of Diapo's `diapo`: the same configuration, sign-in, output and sync rules,
so what is true of one is true of the other.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/Clap/main/install.sh | bash
```

Or `kit add clap` when you already have [kit](https://github.com/FacileStudio/kit).

The first binaries are published with the first tagged release. Until then, run the CLI from a
checkout:
`cargo run -p clap-cli -- <command>`.

`clap render` needs `chrome-headless-shell` and `ffmpeg` on the machine. Every other command
needs nothing.

## Commands

| Command | Does |
|---|---|
| `clap new <name> [-t starter]` | Creates the folder `<name>` with a `clap.md` and an empty `assets/`: the welcome scene in your language, or the starter `-t` names |
| `clap starters` | Lists the starters: name, seconds, what it is |
| `clap serve [dir] [-p port]` | Serves a player around the project on `http://localhost:3030` and reloads it on every change |
| `clap check [dir]` | Prints every problem as `file:line: message` and exits 1 when there is one |
| `clap guide` | Prints the format reference: the scene format, Clap's section, the command line's |
| `clap timeline [dir]` | Prints the scenes, tracks and clips |
| `clap edit [dir] --ops <json\|@file>` | Applies timeline operations to `clap.md` |
| `clap render [dir] [-o file]` | Renders an MP4 on this machine. `--remote` renders on the server |
| `clap login [server]` | Signs in and stores the token |
| `clap logout` | Ends the session and removes the stored token |
| `clap whoami` | Prints the signed-in account |
| `clap list` | Lists your projects, id then title. `-q` prints ids only |
| `clap pull [project] [dir]` | Downloads a project and its assets. `project` is an id or the project's web URL |
| `clap push [dir]` | Uploads the folder's source and assets. `--create` makes a new project |
| `clap open [dir]` | Prints the project's web URL and opens it. `--print` only prints |

`[dir]` is a project folder or a `.md` file, and defaults to the current folder.

Every command takes three global flags:

| Flag | Does |
|---|---|
| `--json` | Prints one JSON document on stdout. Status lines move to stderr |
| `--no-color` | Disables color. `NO_COLOR` does the same |
| `--server <url>` | Uses this server instead of the stored one |

Exit codes: `0` success, `1` error, `2` usage error, `130` on Ctrl+C.

With `--json`, a failed command prints `{"error":{"code","message"}}` on stdout. The codes a
script can match: `conflict`, `local_changes`, `not_linked`, `not_logged_in`, `unauthorized`,
`unreachable`, `invalid`, `asset_refused`, `asset_too_large`, `no_project`, `unknown_starter`,
`bad_ops`, `unknown_clip`, `unknown_scene`, `overlap`, `out_of_bounds`, `invalid_op`,
`invalid_result`, `missing_tool`, `bad_range`, `render_failed`, `render_refused`,
`missing_assets`, `queue_full`, `interrupted`, and any `error.code` the server sends.

## A local project

```sh
clap new launch --template lower-third
clap serve launch
clap render launch          # writes launch.mp4
```

A project is a folder: `clap.md`, and its media under `assets/`. The source names a file by its
path from the folder, such as `src="assets/shot.mp4"`.

Without `--template`, `new` writes the welcome scene the API gives a new project, in the language
of `LC_ALL`, `LC_MESSAGES` or `LANG` (French, English, German or Spanish; French otherwise).
`clap starters` lists what `--template` takes: `title`, `lower-third`, `caption`, `chart-race`,
`logo-reveal`. They are the files of `starters/`, built into the binary.

`check` validates with the scene parser, reads the timeline, and reports a file the project
names but the folder lacks, with the line that names it. `check --json` answers
`{"file","valid","errors":[{"line","code","message"}],"timeline":{"scenes","clips","duration"}}`.

### Preview

`serve` answers a player page: the project in a sandboxed frame (`sandbox="allow-scripts"`),
with a play button, a scrubber and a timecode. Space plays and pauses. The frame holds the
compiled project plus Clap's media runtime, so a `<video>` or `<audio>` clip follows the
scrubber from its `data-in`. Files are served with byte ranges, which a browser needs to seek
a video. The preview plays media in the browser and is not frame-exact: the render is.

A problem in the file is printed in the terminal as `launch/clap.md:12: message` and shown in
the frame as an overlay over the last project that compiled. The player keeps its time across
a reload.

### Timeline and edits

```sh
clap timeline launch
clap timeline launch --json
clap edit launch --ops '[{"op":"move_clip","clip":"st-third","start":1},{"op":"set_scene_duration","scene":0,"duration":6}]'
clap edit launch --ops @ops.json      # or @- for standard input
```

`timeline --json` prints the model of [timeline.md](timeline.md). `edit` takes the same
operations as the API's `POST /projects/{id}/edits` and the `clap_timeline_edit` tool: a JSON
array, or one object. They apply in order as minimal text patches. The file is written once,
when every operation applied; a refused one changes nothing and exits 1 with its code.

### Render

```sh
clap render launch -o launch.mp4 --width 1280 --height 720 --fps 30 --range 0:4
```

The size defaults to the project's `size`, the rate to 30, the range to the whole project.
`render` compiles the project into a hidden folder inside the project folder, links the files
it names beside it, adds Clap's stage stylesheet as the preview has it, and hands that folder to
`clap-render` ([rendering.md](rendering.md)): the
page loads nothing else, and the folder is removed when the render ends. A progress line shows
the frames. Ctrl+C stops at the next frame, closes Chrome and ffmpeg, removes the partial file
and exits 130.

`render` finds `chrome-headless-shell` through `CLAP_CHROME`, then `PATH`, then Playwright's
cache, and `ffmpeg` through `CLAP_FFMPEG`, then `PATH`. When one is missing it names the
variable to set. `npx playwright install chromium-headless-shell` installs the first.

`render --remote` needs a pushed project. It queues `POST /api/projects/{id}/renders` with the
flags that were given, polls `GET /api/renders/{id}` once a second, and downloads
`GET /api/renders/{id}/file` to `-o`. The server renders what was last pushed: the command warns
when the folder changed since. Ctrl+C asks the server to stop the render.

## Signing in

`clap login` asks the server how it signs people in (`GET /api/auth/config`), then runs one
of two flows from porte:

- **Browser redirect** (the default). The CLI listens on a free port of `127.0.0.1`, opens
  `/api/auth/oidc?flow=cli&port=<port>&cli_state=<nonce>`, and waits for the redirect. It
  takes only the callback that carries its own nonce, then trades the one-time code at
  `POST /api/auth/oidc/exchange`.
- **Device grant**, with `--device`, and by default on a Linux machine with no display. The
  CLI starts an RFC 8628 device sign-in at the identity provider with the `facile-cli` client,
  prints a link to approve from any browser, then trades the provider's access token at
  `POST /api/auth/oidc/device/exchange`. That token is never stored. The server mounts this
  route only when `OIDC_CLI_AUDIENCE` is set; `--device` says so when it is not.

`--no-browser` prints the link and opens nothing. `clap logout` calls `POST /api/auth/logout`
and removes the token; it is not an error when there is none.

## Configuration

The CLI stores its login in `${XDG_CONFIG_HOME:-~/.config}/clap/config.yml`, on every
platform:

```yaml
server_url: https://clap.facile.studio
token: <session token>
```

The file is created with mode `0600` in a `0700` folder, and tightened to `0600` when it is
read with wider permissions. `login` and `logout` rewrite only their own two lines.

| Variable | Overrides |
|---|---|
| `CLAP_TOKEN` | The stored token |
| `CLAP_SERVER_URL` | The stored server |
| `CLAP_ISSUER` | The identity provider of the device grant (default `https://sso.facile.studio`) |
| `CLAP_CHROME`, `CLAP_FFMPEG` | The binaries `clap render` runs |

Precedence: flag, then environment, then the config file, then `https://clap.facile.studio`.
The stored token is sent only to the server that issued it: with another `--server` or
`CLAP_SERVER_URL`, set `CLAP_TOKEN` too. A server may be written with or without `/api`.

## Syncing with a server

```sh
clap pull https://clap.facile.studio/projects/<id>    # or: clap pull <id> [dir]
clap push                                             # from inside the folder
```

A pulled folder holds a `.clap.json` that links it to its project:

```json
{
  "project": "<id>",
  "server": "https://clap.facile.studio",
  "rev": 7,
  "source_sha256": "…",
  "assets": { "assets/shot.mp4": "…" }
}
```

`rev`, `source_sha256` and `assets` record what both sides held at the last pull or push. Every
decision compares three states: that record, the folder, and the server.

**Push** validates the project, then sends the source with the stored `rev`
(`PUT /api/projects/{id}/source`). When the project changed on the server the API answers
`409`: the push stops before it writes anything, exits 1 with the code `conflict`, and names
the two ways out, `clap pull` or `clap push --force`. `--force` reads the current `rev` first,
then overwrites. A source the server refuses (`422`) is printed as `file:line: message`.
`push --create` makes a new personal project from the folder (`POST /api/projects` with the
source). After the source, assets are synced by SHA-256:

| Asset | Push does |
|---|---|
| In the folder, absent or different on the server | Uploads it |
| On the server, deleted from the folder since the last sync | Deletes it on the server |
| On the server, never in the folder | Leaves it and names it |

**Pull** refuses to overwrite `clap.md`, or an asset, that changed in the folder since the
last sync. `--force` takes the server's copy. It removes a local asset only when the server
dropped it and the local file is unchanged.

Assets are the files under `assets/`, named as the source writes them (`assets/shot.mp4`). A
hidden file is never synced. Media are large, so nothing is held in memory: a file is hashed
in pieces, uploaded as it is read from disk, and downloaded into a `.part` file that is
renamed when complete. Each transfer shows its progress in MiB and has no time limit.

The server types each file by its bytes and takes only video, audio, images and fonts, with
names made of ASCII letters, digits, `.`, `_` and `-`. By default it takes 2 GiB per file and
20 GiB for all the projects of one space. A refused upload stops the push, names the file,
gives the server's reason, and for a `413` those two caps. A pull also fetches an asset the
server holds outside `assets/`, and never writes a name that leaves the folder, is hidden, or
is `clap.md`.

## The API it calls

All HTTP lives in `apps/cli/src/api`. The base is `<server>/api`, with a bearer token.

| Call | Used by |
|---|---|
| `GET /auth/config`, `POST /auth/oidc/exchange`, `POST /auth/oidc/device/exchange` | `login` |
| `GET /auth/me`, `POST /auth/logout` | `whoami`, `login`, `logout` |
| `GET /projects`, `POST /projects` | `list`, `push --create` |
| `GET /projects/{id}`, `PUT /projects/{id}/source` | `pull`, `push` |
| `GET /projects/{id}/assets`, `POST /projects/{id}/assets` (multipart, parts `name` and `file`) | `pull`, `push` |
| `GET /projects/{id}/assets/{name}`, `DELETE /projects/{id}/assets/{name}` | `pull`, `push` |
| `POST /projects/{id}/renders`, `GET /renders/{id}`, `GET /renders/{id}/file`, `DELETE /renders/{id}` | `render --remote` |

The tests run every one of these against a mock server in the test process. None has run
against a deployed Clap yet.

## Releases

A `v*` tag runs `.github/workflows/release.yml`, which publishes:

```
clap_<version>_darwin_amd64.tar.gz
clap_<version>_darwin_arm64.tar.gz
clap_<version>_linux_amd64.tar.gz     static, musl
clap_<version>_linux_arm64.tar.gz     static, musl
checksums.txt
```

Each archive holds the `clap` binary at its root. `clap --version` prints the version of
`apps/cli/Cargo.toml`, so that version must equal the tag: bump it, with
`apps/client/package.json`, in the tagged commit. The gate and the workflow both check it.
No tag has been pushed yet.

## The `kit` catalog entry

The block in `crates/kit-core/catalog/tools.yml` in `FacileStudio/kit`. Every `auth` field is
transcribed from the code in `apps/cli/src/config` and `apps/cli/src/api/auth.rs`:

```yaml
  - name: clap
    summary: Video as one text file
    repo: FacileStudio/Clap
    branch: main
    bin: clap
    build: rust
    srcSubdir: apps/cli
    asset: clap
    auth:
      kind: oidc-device
      defaultServerUrl: https://clap.facile.studio
      apiSuffix: /api
      discoveryPath: /auth/config
      identityPath: /auth/me
      transport: bearer
      envToken: CLAP_TOKEN
      envUrl: CLAP_SERVER_URL
      oidcDevice:
        issuer: https://sso.facile.studio
        clientId: facile-cli
        scopes: openid profile email
        exchangePath: /auth/oidc/device/exchange
      sso:
        startPath: /auth/oidc
        portParam: port
        stateParam: cli_state
        extraParams: flow=cli
        callbackWith: code
        callbackPath: /
        exchangePath: /auth/oidc/exchange
        requireState: true
      store:
        kind: file
        path: ${xdgConfig}/clap/config.yml
        format: yaml
        tokenField: token
        urlField: server_url
        mode: 0o600
        dirMode: 0o700
        preserve: true
```

Three notes for whoever adds it:

- `facile login` stores the URL with `apiSuffix`, as `https://clap.facile.studio/api`. The CLI
  reads a server with or without `/api`, so both spellings work.
- `srcSubdir: apps/cli` is a workspace member. `apps/cli/.cargo/config.toml` sets the target
  folder to `apps/cli/target` for a build run from there, which is where the installer's
  source build looks for the binary.
- `clap render` needs `chrome-headless-shell` and `ffmpeg`. They are not in `requires:` because
  every other command works without them and the command says what is missing.

Back to the [documentation index](README.md).
