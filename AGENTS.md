# AGENTS.md

**alias** — Git-like aliases for any CLI program. A wrapper binary replaces the target executable, expands aliases, and forwards arguments. Rust 2024, MSRV 1.85, single binary. `make help` lists every command.

## Definition of Done
Done = `make check` green (locally and on Linux, Windows, macOS CI) + a `tests/cli.rs` E2E test that demonstrates the feature. Code written is not done.

One feature at a time: the next starts after the current one is done. Refactoring waits for done.

## Invariants
- **Terminating**: `executable` pointing at the wrapper itself is an error before anything runs. Recursion through a shell alias stops at `ALIAS_DEPTH` (ceiling 16).
- **Portable**: platform-specific code lives behind `#[cfg]`.
- **Std-only**: `toml` is the sole dependency; processes and environment go through `std::process` and `std::env`.
- **Shell aliases (`!`)** need `SHELL` set (Windows: Git Bash, MSYS2, WSL). Absent `SHELL` yields an error message.
- **Arguments** split like git: `""` and `''` keep spaces inside one argument.
- **Config** is TOML: `config.toml` plus optional `override.toml`, both next to the wrapper binary.
- **`${VAR}`** in `executable` is expanded.

## Tests
Every test is E2E: an isolated directory, the wrapper binary copied under the target program's name, a `config.toml` beside it, the wrapper run as a separate process. The wrapper's behavior shows only in the target's argv, so assert there. Unit tests are rare.

Binary setup takes the write lock on `EXECUTABLES: RwLock<()>` (guards the Linux fork/exec ETXTBSY race); running a wrapper takes the read lock.

## Where to look
- `tests/cli.rs` — source of truth for behavior; read before changing any
- `src/main.rs` — `get_handler()` routes to `--aliases`, `--version`, `--help`, default, error handlers
- `src/handler/default.rs` — alias expansion and target invocation
- `src/config/mod.rs` — TOML parsing, alias resolution, nested groups
- `src/environment/mod.rs` — args, env vars, target auto-detection
- `src/process/mod.rs` — spawning (regular / shell / dry run), nesting limit
- `docs/sample_config.toml` — annotated config; `README.md` — user docs

Handlers are stateless: config and environment arrive as `handle()` arguments.
