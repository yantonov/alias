# AGENTS.md

## Project
**alias** — Git-like aliases for any CLI program.

A thin wrapper that replaces the target executable: it intercepts calls, expands aliases, and forwards arguments to the target program. Configuration lives in `config.toml` (and an optional `override.toml`), placed next to the wrapper binary.

Rust, edition 2024, MSRV 1.85. Built with Cargo, shipped as a single binary with no external runtime dependencies.

## Commands
- Install toolchain: `rustup default stable`
- Build: `cargo build`
- Tests: `cargo test`
- Release build (size-optimized): `cargo build --release`
- Lint + format: `cargo clippy --all-targets -- -D warnings && cargo fmt --check`
- Full verification (tests + lint + format): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`

## Hard constraints (MUST)
- The wrapper must never call itself endlessly. Loops are detected: if `executable` points at the wrapper itself — an error before anything runs; if recursion happens through a shell alias — depth is capped by `ALIAS_DEPTH` (ceiling 16).
- Platform-specific code only through `#[cfg]`, never through assumptions. Code compiles on Linux, Windows, and macOS.
- No external runtime dependencies beyond `toml` (config parsing). No direct system calls — only through `std::process` and `std::env`.
- Shell aliases (`!`) run only when the `SHELL` environment variable is set. On Windows that means Git Bash, MSYS2, or WSL. If `SHELL` is absent — an error message, not a crash.
- Alias arguments are split the same way git splits its own: `""` and `''` keep spaces inside a single argument.
- Configuration is TOML. `config.toml` and `override.toml` sit in the same directory as the wrapper binary.
- Environment variables in `executable` are expanded (`${VAR}` syntax).

## Definition of Done
A feature is done = `cargo test` is green on all three platforms in CI + `cargo clippy` and `cargo fmt` are clean + `tests/cli.rs` has an E2E test that demonstrates the feature by running the wrapper as a separate process.

"Code written" is not done.

## Work rules
- One feature at a time. Do not start a second before the first passes verification.
- No "incidental" refactoring while the main feature is unverified.
- Tests are always integration (E2E): each test case creates an isolated directory, copies the wrapper binary under the target program's name, places a `config.toml` next to it, and runs the wrapper as a separate process. Unit tests are rare — the wrapper's behavior only manifests in what ends up in the target program's argv.
- Write-lock on binary setup: `EXECUTABLES: RwLock<()>` guards against the fork/exec race on Linux (ETXTBSY). Tests that set up the wrapper take the write lock; tests that run it share the read lock.
- Before ending a session, make sure all tests pass.

## Where to find details
- `src/main.rs` — entry point, handler routing
- `src/config/mod.rs` — TOML config parsing, alias resolution (including nested groups)
- `src/environment/mod.rs` — environment variables, CLI arguments, auto-detection of the target executable
- `src/handler/default.rs` — core logic: alias expansion and target program invocation
- `src/process/mod.rs` — process spawning (regular/shell), nesting limit check, dry run
- `tests/cli.rs` — all E2E tests; the primary source of truth for wrapper behavior
- `docs/sample_config.toml` — annotated configuration example
- `README.md` — user-facing documentation

## Architecture
```
main.rs → get_handler() → Handler::handle()
                │
    ┌───────────┼───────────┬──────────┬───────────┐
    ▼           ▼           ▼          ▼           ▼
--aliases   --version   --help    default    error
(AliasList) (Version)   (Help)   (Default)   (Error)
                                      │
                              resolve alias in config
                                      │
                              process::execute()
                              (regular / shell / dry run)
```

The config is loaded once at startup from `config.toml` (and optionally merged with `override.toml`). The environment is collected from `std::env::args()` and environment variables. Handlers are stateless: config and environment are passed into `handle()`.

## Project state
- Current version: 0.3.0
- Tests cover: all alias types (regular, shell), groups and nested groups, dry run, loop detection, nesting limit, quoted arguments, env variables in `executable`, auto-detection of the target binary, shell aliases without `SHELL`
- Build targets: linux (x86_64, aarch64), windows (x86_64), macos (arm64, x86_64)
