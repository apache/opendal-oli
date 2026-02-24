---
name: oli
description: Direct command playbooks for Apache OpenDAL Oli CLI data access and mutation. Use when Codex must run real `oli` commands to list, read, copy, move, delete, edit, stream, or benchmark data across local paths and `profile:/path` locations.
---

# Apache OpenDAL Oli

## Command Principles

- Prefer direct command execution over abstract planning.
- Run read-only checks before mutating data.
- Use explicit paths and explicit profile source selection (config-file mode or environment mode).

## Direct Command Playbooks

### Inspect Data Without Mutation

```bash
oli ls 's3:/warehouse/'
oli ls -r 's3:/warehouse/2026/'
oli ls -T 's3:/warehouse/'
oli stat 's3:/warehouse/events.parquet'
oli cat 's3:/warehouse/schema.json'
```

### Copy Local File To Remote

```bash
oli cp ./events.parquet 's3:/ingest/events.parquet'
oli stat 's3:/ingest/events.parquet'
```

### Copy Remote File To Local

```bash
oli cp 's3:/ingest/events.parquet' ./events.parquet
```

### Copy Across Profiles

```bash
oli cp 's3:/warehouse/events.parquet' 'r2:/backup/events.parquet'
oli stat 'r2:/backup/events.parquet'
```

### Recursive Directory Copy

```bash
oli ls -r 's3:/warehouse/2026/'
oli cp -r 's3:/warehouse/2026/' 'r2:/backup/2026/'
oli ls -r 'r2:/backup/2026/'
```

### Move File Or Directory

```bash
oli mv 's3:/staging/events.parquet' 's3:/warehouse/events.parquet'
oli mv -r 's3:/staging/old-batch/' 's3:/archive/old-batch/'
```

### Safe Recursive Delete

```bash
oli ls -r 's3:/staging/to-delete/'
oli rm -r 's3:/staging/to-delete/'
oli ls 's3:/staging/'
```

### Stream Input To Object

```bash
printf '%s\n' '{"status":"ok"}' | oli tee 's3:/logs/healthcheck.json'
```

### Edit Remote File

```bash
EDITOR=vim oli edit 's3:/docs/README.md'
```

### Run Benchmark Suite

```bash
oli bench --config ./config.toml s3 ./suite.toml
```

## Hard Rules

- Always use `profile:/path` syntax and avoid `://`.
- Use `profile:/path` for remote targets and plain paths for local filesystem targets.
- For directory-level mutations, explicitly use `-r` and inspect scope first.
- For delete operations, list first, delete second, verify third.
- For cross-profile copy and move, verify source and destination with `stat` or `ls`.
- In config-file mode, pass `--config <path>` explicitly.
- In environment-variable mode, do not pass `--config`.

## Failure Triage

- `unknown profile`: check profile name and `config.toml`.
- `missing 'type' in profile`: add profile `type`.
- `Host part in a location is not supported`: replace `://` with `:/`.
- Recursive command errors: confirm `-r` and target kind (file vs directory).

## Resource Navigation

- Read `references/profile-setup.md` for config-file mode vs environment-variable mode.
- Read `references/commands.md` for command semantics and flags.
- Read `references/safety-troubleshooting.md` for risk control and debugging.
