# protoc-gen-seaorm

A protoc plugin that generates [SeaORM 2.0](https://docs.rs/sea-orm/2.0.0-rc.21/sea_orm/) entity models from Protocol Buffer definitions.

## Overview

This plugin allows you to define your database schema in `.proto` files using custom `(seaorm.*)` annotations, then generate Rust SeaORM entities automatically. It targets the SeaORM 2.0 dense entity format.

### Example

**Input** (`user.proto`):
```protobuf
syntax = "proto3";
package models;

import "seaorm/options.proto";

message User {
  option (seaorm.message).table_name = "users";

  int64 id = 1 [(seaorm.field) = { primary_key: true, auto_increment: true }];
  string email = 2 [(seaorm.field).unique = true];
  string name = 3;
}
```

**Output** (`user.rs`):
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub email: String,
    pub name: String,
}
```

## Architecture

```
┌─────────────┐     stdin      ┌──────────────────────┐     stdout     ┌─────────────┐
│   protoc    │ ───────────▶   │  protoc-gen-seaorm   │ ─────────────▶ │  .rs files  │
│             │  (protobuf)    │                      │   (protobuf)   │             │
└─────────────┘                └──────────────────────┘                └─────────────┘
                                         │
                                         ▼
                               ┌──────────────────────┐
                               │  CodeGeneratorRequest │
                               │  - file_to_generate  │
                               │  - proto_file[]      │
                               │  - parameter         │
                               └──────────────────────┘
                                         │
                                         ▼
                               ┌──────────────────────┐
                               │  For each message:   │
                               │  1. Parse options    │
                               │  2. Map types        │
                               │  3. Generate code    │
                               └──────────────────────┘
                                         │
                                         ▼
                               ┌──────────────────────┐
                               │ CodeGeneratorResponse │
                               │  - file[] (name,     │
                               │           content)   │
                               └──────────────────────┘
```

## Project Structure

```
protoc-gen-seaorm/
├── Cargo.toml
├── build.rs                      # Compiles proto/seaorm/options.proto
├── proto/
│   └── seaorm/
│       └── options.proto         # Custom (seaorm.*) extension definitions
├── src/
│   ├── main.rs                   # Plugin entry: stdin → process → stdout
│   ├── lib.rs                    # Library interface
│   ├── generator.rs              # Orchestrates code generation
│   ├── options.rs                # Parses (seaorm.*) extensions from descriptors
│   ├── types.rs                  # Proto type → Rust/SeaORM type mapping
│   └── codegen/
│       ├── mod.rs
│       ├── entity.rs             # Generates Model struct
│       ├── column.rs             # Generates column attributes
│       └── relation.rs           # Generates relation fields
└── tests/
    ├── fixtures/                 # Sample .proto files for testing
    └── integration.rs
```

## Key Concepts

### Custom Protobuf Extensions

We define custom options in `proto/seaorm/options.proto` using protobuf's extension mechanism:

- `(seaorm.message)` - Message-level options (table_name, indexes, skip)
- `(seaorm.field)` - Field-level options (primary_key, column_type, relations, etc.)

Extension field numbers 50000-50001 are in the reserved range for custom options.

### Type Mapping

Proto types are mapped to SeaORM types with smart inference:

| Proto Type | SeaORM/Rust Type |
|------------|------------------|
| `int32/int64` | `i32/i64` |
| `string` | `String` |
| `bool` | `bool` |
| `bytes` | `Vec<u8>` |
| `google.protobuf.Timestamp` | `DateTimeUtc` |
| Enum types | Generated SeaORM enum |

Explicit override via `(seaorm.field).column_type = "Uuid"`.

### Relations

Supported relation types:
- `has_one` - One-to-one relationship
- `has_many` - One-to-many relationship
- `belongs_to` - Foreign key relationship (with `from`/`to` columns)
- `has_many_via` - Many-to-many through junction table

### Extension Parsing Challenge

prost doesn't natively support custom extensions. We handle this by:
1. Compiling our `options.proto` to Rust types via `prost-build`
2. Extracting extension data from `uninterpreted_option` fields or raw wire format
3. Deserializing into our `seaorm::MessageOptions` and `seaorm::FieldOptions` types

## Usage

### With protoc

```bash
protoc --seaorm_out=./src/entities \
       -I proto \
       proto/models/*.proto
```

### With buf

```yaml
# buf.gen.yaml
version: v2
plugins:
  - local: protoc-gen-seaorm
    out: src/entities
```

```bash
buf generate
```

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Running manually

```bash
# Generate a CodeGeneratorRequest, pipe through plugin
protoc --seaorm_out=. -I proto proto/test.proto
```

## Dependencies

- `prost` / `prost-types` - Protobuf types and encoding
- `quote` / `proc-macro2` - Rust code generation
- `heck` - Case conversion (snake_case, PascalCase)
- `thiserror` - Error handling

## Target SeaORM Version

This plugin targets **SeaORM 2.0** (currently 2.0.0-rc.21) and generates the new dense entity format with inline relations.

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Auto-syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
bd ready --json
```

**Create new issues:**
```bash
bd create "Issue title" -t bug|feature|task -p 0-4 --json
bd create "Issue title" -p 1 --deps discovered-from:bd-123 --json
bd create "Subtask" --parent <epic-id> --json  # Hierarchical subtask (gets ID like epic-id.1)
```

**Claim and update:**
```bash
bd update bd-42 --status in_progress --json
bd update bd-42 --priority 1 --json
```

**Complete work:**
```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task**: `bd update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`
6. **Commit together**: Always commit the `.beads/issues.jsonl` file together with the code changes so issue state stays in sync with code state

### Auto-Sync

bd automatically syncs with git:
- Exports to `.beads/issues.jsonl` after changes (5s debounce)
- Imports from JSONL when newer (e.g., after `git pull`)
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ✅ Run `bd <cmd> --help` to discover available flags
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

### Using bv as an AI sidecar

bv is a graph-aware triage engine for Beads projects (.beads/beads.jsonl). Instead of parsing JSONL or hallucinating graph traversal, use robot flags for deterministic, dependency-aware outputs with precomputed metrics (PageRank, betweenness, critical path, cycles, HITS, eigenvector, k-core).

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). For agent-to-agent coordination (messaging, work claiming, file reservations), use [MCP Agent Mail](https://github.com/Dicklesworthstone/mcp_agent_mail).

**⚠️ CRITICAL: Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.**

#### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns everything you need in one call:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command

#### Other Commands

**Planning:**
| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with `unblocks` lists |
| `--robot-priority` | Priority misalignment detection with confidence |

**Graph Analysis:**
| Command | Returns |
|---------|---------|
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS (hubs/authorities), eigenvector, critical path, cycles, k-core, articulation points, slack |
| `--robot-label-health` | Per-label health: `health_level` (healthy\|warning\|critical), `velocity_score`, `staleness`, `blocked_count` |
| `--robot-label-flow` | Cross-label dependency: `flow_matrix`, `dependencies`, `bottleneck_labels` |
| `--robot-label-attention [--attention-limit=N]` | Attention-ranked labels by: (pagerank × staleness × block_impact) / velocity |

**History & Change Tracking:**
| Command | Returns |
|---------|---------|
| `--robot-history` | Bead-to-commit correlations: `stats`, `histories` (per-bead events/commits/milestones), `commit_index` |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues, cycles introduced/resolved |

**Other Commands:**
| Command | Returns |
|---------|---------|
| `--robot-burndown <sprint>` | Sprint burndown, scope changes, at-risk items |
| `--robot-forecast <id\|all>` | ETA predictions with dependency-aware scheduling |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions, cycle breaks |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |
| `--export-graph <file.html>` | Self-contained interactive HTML visualization |

#### Scoping & Filtering

bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work (no blockers)
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank scores
bv --robot-triage --robot-triage-by-track    # Group by parallel work streams
bv --robot-triage --robot-triage-by-label    # Group by domain

#### Understanding Robot Output

**All robot JSON includes:**
- `data_hash` — Fingerprint of source beads.jsonl (verify consistency across calls)
- `status` — Per-metric state: `computed|approx|timeout|skipped` + elapsed ms
- `as_of` / `as_of_commit` — Present when using `--as-of`; contains ref and resolved SHA

**Two-phase analysis:**
- **Phase 1 (instant):** degree, topo sort, density — always available immediately
- **Phase 2 (async, 500ms timeout):** PageRank, betweenness, HITS, eigenvector, cycles — check `status` flags

**For large graphs (>500 nodes):** Some metrics may be approximated or skipped. Always check `status`.

#### jq Quick Reference

bv --robot-triage | jq '.quick_ref'                        # At-a-glance summary
bv --robot-triage | jq '.recommendations[0]'               # Top recommendation
bv --robot-plan | jq '.plan.summary.highest_impact'        # Best unblock target
bv --robot-insights | jq '.status'                         # Check metric readiness
bv --robot-insights | jq '.Cycles'                         # Circular deps (must fix!)
bv --robot-label-health | jq '.results.labels[] | select(.health_level == "critical")'

**Performance:** Phase 1 instant, Phase 2 async (500ms timeout). Prefer `--robot-plan` over `--robot-insights` when speed matters. Results cached by data hash.

Use bv instead of parsing beads.jsonl—it computes PageRank, critical paths, cycles, and parallel tracks deterministically.
