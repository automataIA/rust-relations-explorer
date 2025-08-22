# rust-relations-explorer: Command Examples

This document shows practical CLI examples using this repository as the project root. The CLI auto-detects the project root (nearest ancestor with `Cargo.toml` + `src/`), so `--path .` is usually unnecessary.

Notes
- All commands assume you run them from the repository root.
- Outputs like JSON/DOT/SVG will be written to the current directory unless a path is specified.
- Many queries can also read an already saved graph via `--graph graph.json` to avoid rebuilding.

## Build Commands

```bash
# 1) Build with cache (default) and export multiple artifacts
rust-relations-explorer build \
  --json graph.json \
  --dot graph.dot \
  --svg graph.svg \
  --dot-rankdir LR --dot-splines curved --dot-rounded on \
  --dot-theme light --dot-clusters on --dot-legend on

# 2) Build forcing a full re-parse without using cache
rust-relations-explorer build --no-cache

# 3) Rebuild cache from scratch (clears previous cache file then parses)
rust-relations-explorer build --rebuild

# 4) Apply options from a config file and also export SVG
rust-relations-explorer build --config rre.toml --svg graph.svg

# 5) Bypass ignore rules (include files even if ignored)
rust-relations-explorer build --no-ignore
```

## Query Commands (building on-the-fly)

```bash
# Connected files for a given file under src/ (positional <file>)
rust-relations-explorer query connected-files src/query/mod.rs -f text

# Function usage: who calls `foo` (callers)
rust-relations-explorer query function-usage --function foo --direction callers -f json

# Function usage: what does `foo` call (callees)
rust-relations-explorer query function-usage --function foo --direction callees -f text

# Detect cycles (file-level projection of call graph)
rust-relations-explorer query cycles -f text

# Shortest path between files (directed)
rust-relations-explorer query path --from src/query/mod.rs --to src/graph/mod.rs -f text

# Hubs: top-N by degree centrality (total, in, out)
rust-relations-explorer query hubs --metric total --top 10 --offset 0 --limit 10 -f text

# Module centrality: top-N modules (directories) by degree centrality
rust-relations-explorer query module-centrality --metric out --top 5 --offset 0 --limit 5 -f json

# Trait implementations: list types implementing a trait name
rust-relations-explorer query trait-impls --trait Display --offset 0 --limit 50 -f text

# Item info by name (name-only lookup); optional kind can disambiguate
rust-relations-explorer query item-info -n resolver -f text

# Quiet mode: suppress non-essential logs
rust-relations-explorer -q query hubs --metric total --top 10 -f json

# Verbosity: increase detail with -v/-vv
rust-relations-explorer -vv query connected-files src/query/mod.rs -f text
```

## Query Commands (using a saved graph)

```bash
# Build once, then query repeatedly using the saved graph
rust-relations-explorer build --json graph.json

# Connected files (uses saved graph instead of rebuilding)
rust-relations-explorer query connected-files --graph graph.json src/query/mod.rs -f text

# Function usage with saved graph
rust-relations-explorer query function-usage --graph graph.json --function foo --direction callers -f json

# Cycles with saved graph
rust-relations-explorer query cycles --graph graph.json -f text

# Shortest path with saved graph
rust-relations-explorer query path --graph graph.json --from src/query/mod.rs --to src/graph/mod.rs -f text

# Hubs and module centrality with saved graph
rust-relations-explorer query hubs --graph graph.json --metric total --top 5 --offset 0 --limit 5 -f text
rust-relations-explorer query module-centrality --graph graph.json --metric in --top 5 --offset 0 --limit 5 -f text

# Trait impls with saved graph
rust-relations-explorer query trait-impls --graph graph.json --trait Display --offset 0 --limit 50 -f json
```

## Advanced/Performance Scenarios

```bash
# Warm run: use cache and export all artifacts
rust-relations-explorer build --json graph.json --dot graph.dot --svg graph.svg

# Cold run: bypass cache to re-parse everything
rust-relations-explorer build --no-cache --json graph.json

# Ignore patterns off: include files typically filtered by .gitignore/.ignore
rust-relations-explorer build --no-ignore --json graph.json
```

## Tips

- Use `--format json` for machine-readable results or `--format text` for tables.
- When iterating on queries, prefer building once and passing `--graph graph.json` to speed up runs.
- For large projects, try a no-cache build occasionally to validate cache correctness.

### Env defaults (optional)

```bash
export RRE_PATH=src
export RRE_FORMAT=text

# With env defaults set, you can omit -p/--path and -f/--format on many commands
rust-relations-explorer query item-info -n resolver
```
