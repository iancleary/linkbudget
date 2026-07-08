# Release process

Use the Forge-managed `create-release-process` skill to maintain this workflow. Use the repo-local `cut-release` flow to execute an ordinary release.

## Versioning

`linkbudget` is a single Rust crate. The version source is the root `Cargo.toml` package version, currently SemVer. The next version is not inferred from repository evidence, so real releases must pass an explicit version:

```bash
just cut-release --version 0.6.2
```

Read-only version queries:

```bash
just cut-release --print-current-version
just cut-release --print-next-version
```

`--print-next-version` intentionally exits non-zero and explains that `--version` is required.

## Dry run

Preview a release without mutating public state:

```bash
just cut-release --dry-run --version 0.6.2
```

The dry run temporarily updates `Cargo.toml` and `Cargo.lock`, runs validation, reports the public commands it would run, and restores the local files before exiting.

## Real release

Run a real release only from a clean `main` branch with GitHub CLI authentication available:

```bash
just cut-release --version 0.6.2
```

The runner owns these local mutations:

- root `Cargo.toml`
- root `Cargo.lock`

It validates with:

- `cargo build --locked --verbose`
- `cargo test --locked --verbose`

Then it commits the version bump, creates an annotated `vX.Y.Z` tag, pushes the branch and tag, and makes `gh release create` the final public-facing release action. The repository's GitHub Actions release workflow publishes to crates.io when the GitHub release is published.

Release notes default to GitHub-generated notes. To use curated notes:

```bash
just cut-release --version 0.6.2 --notes-file path/to/notes.md
```
