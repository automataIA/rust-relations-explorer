# knowledge-rs: Command Examples

This document shows practical CLI examples using this repository as the project path (`--path .`) and files under the local `src/` directory.

Notes
- All commands assume you run them from the repository root.
- Outputs like JSON/DOT/SVG will be written to the current directory unless a path is specified.
- Many queries can also read an already saved graph via `--graph graph.json` to avoid rebuilding.

## Build Commands

```bash
# 1) Build with cache (default) and export multiple artifacts
knowledge-rs build --path . \
  --json graph.json \
  --dot graph.dot \
  --svg graph.svg \
  --dot-rankdir LR --dot-splines curved --dot-rounded on \
  --dot-theme light --dot-clusters on --dot-legend on

# 2) Build forcing a full re-parse without using cache
knowledge-rs build --path . --no-cache

# 3) Rebuild cache from scratch (clears previous cache file then parses)
knowledge-rs build --path . --rebuild

# 4) Apply options from a config file and also export SVG
knowledge-rs build --path . --config knowledge-rs.toml --svg graph.svg

# 5) Bypass ignore rules (include files even if ignored)
knowledge-rs build --path . --no-ignore
```

## Query Commands (building on-the-fly)

```bash
# Connected files for a given file under src/
knowledge-rs query connected-files --path . --file src/query/mod.rs --format text

# Function usage: who calls `foo` (callers)
knowledge-rs query function-usage --path . --function foo --direction callers --format json

# Function usage: what does `foo` call (callees)
knowledge-rs query function-usage --path . --function foo --direction callees --format text

# Detect cycles (file-level projection of call graph)
knowledge-rs query cycles --path . --format text

# Shortest path between files (directed)
knowledge-rs query path --path . --from src/query/mod.rs --to src/graph/mod.rs --format text

# Hubs: top-N by degree centrality (total, in, out)
knowledge-rs query hubs --path . --metric total --top 10 --format text

# Module centrality: top-N modules (directories) by degree centrality
knowledge-rs query module-centrality --path . --metric out --top 5 --format json

# Trait implementations: list types implementing a trait name
knowledge-rs query trait-impls --path . --trait Display --format text
```

## Query Commands (using a saved graph)

```bash
# Build once, then query repeatedly using the saved graph
knowledge-rs build --path . --json graph.json

# Connected files (uses saved graph instead of rebuilding)
knowledge-rs query connected-files --graph graph.json --file src/query/mod.rs --format text

# Function usage with saved graph
knowledge-rs query function-usage --graph graph.json --function foo --direction callers --format json

# Cycles with saved graph
knowledge-rs query cycles --graph graph.json --format text

# Shortest path with saved graph
knowledge-rs query path --graph graph.json --from src/query/mod.rs --to src/graph/mod.rs --format text

# Hubs and module centrality with saved graph
knowledge-rs query hubs --graph graph.json --metric total --top 5 --format text
knowledge-rs query module-centrality --graph graph.json --metric in --top 5 --format text

# Trait impls with saved graph
knowledge-rs query trait-impls --graph graph.json --trait Display --format json
```

## Advanced/Performance Scenarios

```bash
# Warm run: use cache and export all artifacts
knowledge-rs build --path . --json graph.json --dot graph.dot --svg graph.svg

# Cold run: bypass cache to re-parse everything
knowledge-rs build --path . --no-cache --json graph.json

# Ignore patterns off: include files typically filtered by .gitignore/.ignore
knowledge-rs build --path . --no-ignore --json graph.json
```

## Tips

- Use `--format json` for machine-readable results or `--format text` for tables.
- When iterating on queries, prefer building once and passing `--graph graph.json` to speed up runs.
- For large projects, try a no-cache build occasionally to validate cache correctness.
