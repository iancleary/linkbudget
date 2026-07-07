#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/cut-release.sh [--dry-run] --version <semver> [--notes-file <path>]
       scripts/cut-release.sh --print-current-version
       scripts/cut-release.sh --print-next-version

Cuts a release for the root Cargo package.

Options:
  --dry-run                 Run local validation and restore local version edits.
  --version <semver>        Required for real releases; next version is not inferred.
  --notes-file <path>       Use curated GitHub release notes from this file.
  --print-current-version   Print the root Cargo package version and exit.
  --print-next-version      Explain why next-version inference is unavailable.
  -h, --help                Show this help.
USAGE
}

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

package_name="linkbudget"
dry_run=0
version=""
notes_file=""
print_current=0
print_next=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    --version)
      version="${2:-}"
      if [[ -z "$version" ]]; then
        echo "error: --version requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --notes-file)
      notes_file="${2:-}"
      if [[ -z "$notes_file" ]]; then
        echo "error: --notes-file requires a path" >&2
        exit 2
      fi
      shift 2
      ;;
    --print-current-version)
      print_current=1
      shift
      ;;
    --print-next-version)
      print_next=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

current_version="$(
  python3 - <<'PY'
import pathlib
import tomllib

manifest = pathlib.Path("Cargo.toml")
print(tomllib.loads(manifest.read_text())["package"]["version"])
PY
)"

if [[ "$print_current" -eq 1 ]]; then
  echo "$current_version"
  exit 0
fi

if [[ "$print_next" -eq 1 ]]; then
  echo "Next version cannot be inferred safely for linkbudget; pass --version <semver>." >&2
  exit 2
fi

if [[ -z "$version" ]]; then
  echo "error: --version <semver> is required because this repo has no documented next-version policy" >&2
  usage >&2
  exit 2
fi

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "error: version must be SemVer, such as 0.6.2" >&2
  exit 2
fi

if [[ "$version" == "$current_version" ]]; then
  echo "error: version is already $current_version" >&2
  exit 2
fi

if [[ -n "$notes_file" && ! -f "$notes_file" ]]; then
  echo "error: notes file not found: $notes_file" >&2
  exit 2
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree must be clean before cutting a release" >&2
  exit 1
fi

tag="v$version"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  echo "error: local tag already exists: $tag" >&2
  exit 1
fi
if git ls-remote --exit-code --tags origin "$tag" >/dev/null 2>&1; then
  echo "error: remote tag already exists: $tag" >&2
  exit 1
fi

branch="$(git branch --show-current)"
if [[ "$dry_run" -eq 0 && "$branch" != "main" ]]; then
  echo "error: real releases must run from main; current branch is $branch" >&2
  exit 1
fi

if [[ "$dry_run" -eq 0 ]]; then
  gh auth status >/dev/null
fi

tmp_dir=""
restore_dry_run() {
  if [[ -n "$tmp_dir" && -d "$tmp_dir" ]]; then
    cp "$tmp_dir/Cargo.toml" Cargo.toml
    cp "$tmp_dir/Cargo.lock" Cargo.lock
    rm -rf "$tmp_dir"
  fi
}

if [[ "$dry_run" -eq 1 ]]; then
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/cut-release.XXXXXX")"
  cp Cargo.toml "$tmp_dir/Cargo.toml"
  cp Cargo.lock "$tmp_dir/Cargo.lock"
  trap restore_dry_run EXIT
fi

python3 - "$version" <<'PY'
import pathlib
import re
import sys

version = sys.argv[1]
manifest = pathlib.Path("Cargo.toml")
text = manifest.read_text()
updated = re.sub(r'(?m)^version = "[^"]+"$', f'version = "{version}"', text, count=1)
if updated == text:
    raise SystemExit("failed to update package version in Cargo.toml")
manifest.write_text(updated)
PY

python3 - "$package_name" "$version" <<'PY'
import pathlib
import re
import sys

package_name = sys.argv[1]
version = sys.argv[2]
lockfile = pathlib.Path("Cargo.lock")
blocks = re.split(r"(?=^\[\[package\]\]$)", lockfile.read_text(), flags=re.MULTILINE)
updated_blocks = []
changed = False
for block in blocks:
    if re.search(rf'(?m)^name = "{re.escape(package_name)}"$', block):
        new_block, count = re.subn(r'(?m)^version = "[^"]+"$', f'version = "{version}"', block, count=1)
        if count != 1:
            raise SystemExit(f"failed to update {package_name} version in Cargo.lock")
        updated_blocks.append(new_block)
        changed = True
    else:
        updated_blocks.append(block)
if not changed:
    raise SystemExit(f"package {package_name} not found in Cargo.lock")
lockfile.write_text("".join(updated_blocks))
PY

cargo build --locked --verbose
cargo test --locked --verbose

if [[ "$dry_run" -eq 1 ]]; then
  echo "Dry run succeeded for $package_name $tag."
  echo "Would commit Cargo.toml and Cargo.lock, tag $tag, push main and $tag, then create the GitHub release."
  if [[ -n "$notes_file" ]]; then
    echo "Would run: gh release create $tag --verify-tag --notes-file $notes_file"
  else
    echo "Would run: gh release create $tag --verify-tag --generate-notes"
  fi
  exit 0
fi

git add Cargo.toml Cargo.lock
git commit -m "chore: release $tag"
git tag -a "$tag" -m "Release $tag"
git push origin HEAD
git push origin "$tag"

if [[ -n "$notes_file" ]]; then
  gh release create "$tag" --verify-tag --notes-file "$notes_file"
else
  gh release create "$tag" --verify-tag --generate-notes
fi
