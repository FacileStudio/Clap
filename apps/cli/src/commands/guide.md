
## From the command line

The `clap` command line does on a folder what the tools above do on the server.

- `clap check [dir]` validates `clap.md` and prints every problem as `file:line: message`.
- `clap timeline [dir] --json` prints the scenes and clips with their ids, times and tracks.
- `clap edit [dir] --ops '<json>'` applies the same timeline operations to the file, in order.
  `--ops @ops.json` reads them from a file, `--ops @-` from standard input.
- `clap serve [dir]` previews the project with a player. `clap render [dir]` writes an MP4.
- `clap starters` lists ready-made scenes; `clap new <name> --template <starter>` begins with one.
- Media are the files under `assets/`, beside `clap.md`. `clap push` uploads them.
