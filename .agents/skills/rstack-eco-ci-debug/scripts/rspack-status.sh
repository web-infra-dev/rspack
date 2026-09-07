#!/usr/bin/env bash
set -euo pipefail

repo_path="${RSTACK_ECOSYSTEM_CI_PATH:-}"
source_ref="origin/data:rspack.json"
source_set=false

usage() {
  cat <<'EOF'
Usage: rspack-status.sh [--repo <rstack-ecosystem-ci-path>] [rspack-json-path-or-git-ref]
       rspack-status.sh [--repo <rstack-ecosystem-ci-path>] --source <rspack-json-path-or-git-ref>

Print the latest and previous Rspack eco-ci status rows from data/rspack.json.

Options:
  --repo <path>    Local rstack-ecosystem-ci checkout for git refs such as origin/data:rspack.json.
                  Can also be set with RSTACK_ECOSYSTEM_CI_PATH.
  --source <ref>   JSON file path or git ref. Defaults to origin/data:rspack.json.
  -h, --help       Show this help message.

Examples:
  rspack-status.sh --repo <rstack-ecosystem-ci-path>
  rspack-status.sh --repo <rstack-ecosystem-ci-path> origin/data:rspack.json
  rspack-status.sh /tmp/rspack.json
EOF
}

while [[ $# -gt 0 ]]; do
  case "${1}" in
    -h | --help)
      usage
      exit 0
      ;;
    --repo | --ecosystem-ci-repo)
      if [[ $# -lt 2 || -z "${2:-}" ]]; then
        echo "rspack-status.sh: --repo requires a path." >&2
        exit 1
      fi
      repo_path="${2}"
      shift 2
      ;;
    --source)
      if [[ $# -lt 2 || -z "${2:-}" ]]; then
        echo "rspack-status.sh: --source requires a JSON path or git ref." >&2
        exit 1
      fi
      if [[ "${source_set}" == true ]]; then
        echo "rspack-status.sh: only one source can be provided." >&2
        exit 1
      fi
      source_ref="${2}"
      source_set=true
      shift 2
      ;;
    --)
      shift
      if [[ $# -gt 0 ]]; then
        if [[ "${source_set}" == true ]]; then
          echo "rspack-status.sh: only one source can be provided." >&2
          exit 1
        fi
        source_ref="${1}"
        source_set=true
        shift
      fi
      if [[ $# -gt 0 ]]; then
        echo "rspack-status.sh: unexpected extra arguments: $*" >&2
        exit 1
      fi
      ;;
    -*)
      echo "rspack-status.sh: unknown option ${1}." >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ "${source_set}" == true ]]; then
        echo "rspack-status.sh: only one source can be provided." >&2
        exit 1
      fi
      source_ref="${1}"
      source_set=true
      shift
      ;;
  esac
done

if [[ ! -f "${source_ref}" && -n "${repo_path}" && ! -e "${repo_path}/.git" ]]; then
  echo "rspack-status.sh: --repo must point to a local rstack-ecosystem-ci checkout." >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "rspack-status.sh requires node in PATH." >&2
  exit 1
fi

if [[ ! -f "${source_ref}" ]] && ! command -v git >/dev/null 2>&1; then
  echo "rspack-status.sh requires git in PATH when reading a git ref." >&2
  exit 1
fi

read_status_json() {
  if [[ -f "${source_ref}" ]]; then
    cat "${source_ref}"
  else
    local git_args=(git)
    if [[ -n "${repo_path}" ]]; then
      git_args+=(-C "${repo_path}")
    fi

    if ! "${git_args[@]}" show "${source_ref}"; then
      if [[ -n "${repo_path}" ]]; then
        echo "Failed to read ${source_ref} from ${repo_path}." >&2
      else
        echo "Failed to read ${source_ref}. Run from the rstack-ecosystem-ci checkout or pass --repo <rstack-ecosystem-ci-path>." >&2
      fi
      exit 1
    fi
  fi
}

status_json_file="$(mktemp)"
trap 'rm -f "${status_json_file}"' EXIT
read_status_json >"${status_json_file}"

node -e '
const fs = require("node:fs");

let rows;
try {
  rows = JSON.parse(fs.readFileSync(0, "utf8"));
} catch (error) {
  console.error(`Failed to parse rspack status JSON: ${error.message}`);
  process.exit(1);
}

if (!Array.isArray(rows) || rows.length === 0) {
  console.error("Rspack status JSON must be a non-empty array.");
  process.exit(1);
}

const latest = rows[0];
const previous = rows[1];

const fallback = (value, defaultValue = "unknown") => value ?? defaultValue;
const failedSuites = (row) => (row?.suites ?? []).filter((suite) => suite.status !== "success");
const suiteNames = (suites) => suites.map((suite) => fallback(suite.name));
const list = (items) => (items.length > 0 ? items.join(", ") : "none");

function printRun(label, row) {
  if (!row) {
    console.log(`${label}: none`);
    return;
  }

  const failed = failedSuites(row);
  console.log(`${label}:`);
  console.log(`  run: ${fallback(row.workflowRunUrl)}`);
  console.log(`  commit: ${fallback(row.commitSha)}`);
  console.log(`  time: ${fallback(row.commitTimestamp)}`);
  console.log(`  message: ${fallback(row.commitMessage)}`);
  console.log(`  status: ${fallback(row.overallStatus)}`);
  console.log(`  failed suites: ${list(suiteNames(failed))}`);
  for (const suite of failed) {
    console.log(`    - ${fallback(suite.name)}: ${fallback(suite.logUrl, "no job url")}`);
  }
}

function diffSuites(current, base) {
  const currentSet = new Set(suiteNames(failedSuites(current)));
  const baseSet = new Set(suiteNames(failedSuites(base)));

  return {
    added: [...currentSet].filter((name) => !baseSet.has(name)).sort(),
    recovered: [...baseSet].filter((name) => !currentSet.has(name)).sort(),
    unchanged: [...currentSet].filter((name) => baseSet.has(name)).sort(),
  };
}

printRun("latest", latest);
console.log("");
printRun("previous", previous);
console.log("");

const delta = diffSuites(latest, previous);
console.log("delta:");
console.log(`  new failing suites: ${list(delta.added)}`);
console.log(`  recovered suites: ${list(delta.recovered)}`);
console.log(`  unchanged failing suites: ${list(delta.unchanged)}`);
' <"${status_json_file}"
