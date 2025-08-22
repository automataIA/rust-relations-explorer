# CLI Simplification Plan (Bottom‑Up)

This plan organizes the work into macro‑tasks with clear checklists. After completing each macro‑task: update this file (mark checkboxes), and run `cargo check -q`.

## Conventions
- Bold defaults and short flags in help.
- Prefer enums (`ValueEnum`) over free strings.
- Keep output short by default; add `-v/--verbose` to expand.
- Resolve project root automatically (nearest ancestor with `Cargo.toml` + `src/`).
- Config precedence: defaults < config file < env < CLI args.

---

## M1 — Project root auto‑detection (foundation)
Goal: Avoid passing `--path` by inferring the Cargo project root.

Checklist:
- [x] Add helper `utils::project_root::detect()` (walk up ancestors; `Cargo.toml` + `src/`).
- [x] Change `path` in `Commands` and `QueryCommands` to `Option<PathBuf>` with `#[arg(short='p', long, env="RRE_PATH")]`.
- [x] After `Cli::parse()`, normalize `path == "."` to detected root via `effective_path_str()`.
- [x] Log decision at `-v` only; silent on default.
- [x] `cargo check -q`.
- [x] Update plan.md.

---

## M2: CLI ergonomics (short flags, visible aliases, formats)
Goal: Less typing, fewer errors.

Checklist:
- [x] On `Cli`: `#[command(arg_required_else_help = true, propagate_version = true)]`.
- [x] Standardize short flags across subcommands (config, no-ignore, format)
  - `-c/--config`
  - `-n/--no-ignore` with visible aliases: `no-gitignore`, `all`
  - `-f/--format` where applicable
- [x] Replace string parsers with ValueEnum for `format`, `direction`, `metric` (and DOT options where applicable)
  - Rationale: type safety, better UX, auto-completions from clap

### In-progress: ValueEnum migration plan

- [x] Step 1 (this PR): Introduce `OutputFormat` ValueEnum (`text`|`json`) and migrate all query subcommands' `--format` to use it.
  - Keep config schema unchanged (still string). Map config `default_format` string to `OutputFormat` at runtime.
  - Ensure short `-f` remains reserved for `--format` (avoid collisions).
  - Run full test suite; fix any type changes in tests constructing `Cli` directly.
  - Status: cargo check and full test suite passed on 2025-08-22 10:32 CEST.

- [x] Step 2: Introduce ValueEnum for `direction` (`callers`|`callees`).
  - Implemented: `Direction` ValueEnum in `src/cli/mod.rs`, CLI flags updated, `src/app.rs` refactored to pattern-match, tests updated.
  - Status: full test suite passed on 2025-08-22 10:47 CEST.

- [x] Step 3: Introduce ValueEnum for `metric` (`in`|`out`|`total`).
  - Implemented: `CentralityMetricArg` ValueEnum for CLI, mapped to runtime `CentralityMetric` in `src/app.rs`, tests updated.
  - Status: full test suite passed on 2025-08-22 10:50 CEST.

- [x] Step 4: ValueEnum for DOT-related flags where feasible (`theme`, `rankdir`, `splines`, `rounded`, `clusters`, `legend`, `svg_interactive`).
  - Implemented: Added `OnOffArg`, `DotThemeArg`, `DotRankDirArg` (with aliases `LR`/`TB`), `DotSplinesArg`. Updated `Commands::Build` fields in `src/cli/mod.rs` to use them, refactored mapping in `src/app.rs`, and updated tests in `tests/app_run_cli.rs`.
  - Status: full test suite passed on 2025-08-22 11:20 CEST.

Notes:
- Verbose logging for project root detection only at `-v` or higher is in place.
- After each step, run `cargo check` and tests.

- [x] `cargo check -q`.
- [x] Update plan.md.

---

## M3 — Output controls (token‑friendly defaults)
Goal: Short output by default; opt‑in verbosity.

Checklist:
- [x] Add `-q/--quiet`, `-v/--verbose` (count), and `--limit/--offset` to query subcommands.
- [x] Make `--short` implicit when `verbose == 0`; expand fields with `-v/-vv/-vvv`.
- [x] Ensure JSON respects `--limit` and `--offset`.
- [x] Drop heavy fields in JSON unless `-v`.
- [x] Update printing utils to branch on verbosity.
- [x] `cargo check -q` and full test suite passed (2025-08-22 12:35 CEST).
- [x] Update plan.md.

---

## M4 — Positional operands for frequent args
Goal: Faster common commands.

Checklist:
- [x] `Query::ConnectedFiles`: make `file` positional (`<file>`); keep `--file` as alias.
- [x] Adjust help/usage examples accordingly.
- [x] Add minimal validation and clear error on missing file.
- [x] `cargo check -q`.
- [x] Update plan.md.

---

## M5 — Name‑only item lookup
Goal: Allow `ItemInfo` by name without full `item_id`.

Checklist:
- [x] Add flags: `--name/-n <NAME>`, optional `--kind/-k <Kind>` (`ValueEnum` mapping to internal `ItemType`).
- [x] Implement resolver (scope-aware) in app integration:
  - [x] Prefer matches in current crate `src/` and scope.
  - [x] If multiple, return top match or list a few with hint to disambiguate (`--kind`, `--item-id`).
  - Note: implementation now uses `Resolver` for name lookups with kind filtering and path-aware ranking.
- [x] Make `item_id` optional if `--name` is provided; error if neither.
- [x] `cargo check -q` and tests passed (2025-08-22 13:03 CEST).
- [x] Update plan.md.

Next steps for M5:
- [x] Add CLI tests for ItemInfo name lookup: success, ambiguous, not found, with kind (added `tests/cli_iteminfo_name.rs`).
- [x] Optionally refactor to use `src/graph/resolver.rs` for smarter, scope-aware matching and crate preference (done, full test suite passing on 2025-08-22 13:29 CEST).

---

## M6 — Env/config layering
Goal: Avoid retyping via env and optional TOML config.

Checklist:
- [x] Add `#[arg(env = ...)]` to frequent args: `path`, `graph`, `format`, etc. (Added `RRE_GRAPH` and `RRE_FORMAT` across all query subcommands; `RRE_PATH` already present.)
- [x] Support `--config` TOML (optional): load into struct and backfill missing values unless overridden by env/CLI. (Applied to DOT/SVG build options and query default format.)
- [x] Document precedence and example env vars. (README updated with precedence: CLI > env > config > defaults, plus `RRE_PATH`, `RRE_GRAPH`, `RRE_FORMAT` examples.)
- [x] `cargo check -q` and full test suite passed after changes.
- [x] Update plan.md.

Status: Completed on 2025-08-22 13:41 CEST. All tests passing.

---

## M7 — Shell completions (quality‑of‑life)
Goal: Make flags discoverable and faster to use.

Checklist:
- [x] Add `clap_complete`.
- [x] Add `cli completions <shell>` subcommand to print completions (bash/zsh/fish/powershell/nu).
- [x] README snippet for installation paths per shell.
- [x] `cargo check -q`.
- [x] Update plan.md.

Status: Completed on 2025-08-22 13:52 CEST. Subcommand implemented, README updated, cargo check passing.

---

## M8 — Docs & tests
Goal: Stabilize and document new UX.

Checklist:
- [x] Update `README.md` with new usage examples (defaults, name lookup, verbosity).
- [x] Add CLI tests in `tests/` for: root detection, positional file, enums, verbosity levels, name lookup (ambiguous/success).
- [x] `cargo check -q` and `cargo test`.
- [x] Update plan.md.

Status: Completed on 2025-08-22 13:56 CEST. README updated, new CLI tests added, cargo check/test passing.

---

## Notes
- Keep help concise; push details behind `-v`.
- Favor deterministic outputs for scripting; guard non‑deterministic lists with `--limit` and stable ordering.
