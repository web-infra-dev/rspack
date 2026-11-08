#!/usr/bin/env bash

set -euo pipefail

readonly DEFAULT_IMAGE="rspack-perf-valgrind:nightly-2026-04-16"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
readonly DOCKER_INSTALL_TASK="019f7e27-50ba-7ac1-8ab4-3cdc868818fe"

usage() {
  cat <<'EOF'
Run reproducible local Rspack benchmark measurements in Docker-backed
Valgrind simulation mode.

Usage:
  run_local_valgrind.sh build-image [--image IMAGE] [--platform PLATFORM]
  run_local_valgrind.sh prepare --repo PATH
  run_local_valgrind.sh measure --repo PATH --fixtures PATH --bench TARGET \
    --filter FILTER --output PATH [--repeat N] [--image IMAGE] \
    [--platform PLATFORM]

Commands:
  build-image  Build the pinned local measurement image.
  prepare      Run `pnpm run bench:prepare` once in the selected checkout.
  measure      Build and measure one benchmark target/filter, retaining logs.

Defaults:
  IMAGE     rspack-perf-valgrind:nightly-2026-04-16
  PLATFORM  Native Docker architecture (`linux/arm64` or `linux/amd64`)
  REPEAT    2

Prerequisite:
  Docker CLI and Engine must be installed and running. If they are missing,
  follow installation task 019f7e27-50ba-7ac1-8ab4-3cdc868818fe first.

A valid measurement produces a nonzero Callgrind instruction total. The
output directory must not contain prior run data.
EOF
}

fail() {
  echo "error: $*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

require_docker() {
  command -v docker >/dev/null 2>&1 \
    || fail "Docker is not installed; follow installation task $DOCKER_INSTALL_TASK before continuing"
  docker info >/dev/null 2>&1 \
    || fail "Docker Engine is unavailable; start and verify Docker using installation task $DOCKER_INSTALL_TASK"
}

canonical_directory() {
  local directory="$1"
  [[ -d "$directory" ]] || fail "directory does not exist: $directory"
  (cd "$directory" && pwd -P)
}

hash_text() {
  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha256sum | cut -c1-16
  else
    printf '%s' "$1" | shasum -a 256 | cut -c1-16
  fi
}

native_docker_platform() {
  local architecture
  architecture="$(docker info --format '{{.Architecture}}')"
  case "$architecture" in
    amd64|x86_64)
      echo "linux/amd64"
      ;;
    arm64|aarch64)
      echo "linux/arm64"
      ;;
    *)
      fail "unsupported Docker architecture: $architecture"
      ;;
  esac
}

command_name="${1:-}"
[[ -n "$command_name" ]] || {
  usage
  exit 2
}
if [[ "$command_name" == "-h" || "$command_name" == "--help" ]]; then
  usage
  exit 0
fi
shift

image="$DEFAULT_IMAGE"
platform=""
repo=""
fixtures=""
bench_target=""
bench_filter=""
output_directory=""
repeat_count=2

while (($# > 0)); do
  case "$1" in
    --image)
      (($# >= 2)) || fail "--image requires a value"
      image="$2"
      shift 2
      ;;
    --platform)
      (($# >= 2)) || fail "--platform requires a value"
      platform="$2"
      shift 2
      ;;
    --repo)
      (($# >= 2)) || fail "--repo requires a value"
      repo="$2"
      shift 2
      ;;
    --fixtures)
      (($# >= 2)) || fail "--fixtures requires a value"
      fixtures="$2"
      shift 2
      ;;
    --bench)
      (($# >= 2)) || fail "--bench requires a value"
      bench_target="$2"
      shift 2
      ;;
    --filter)
      (($# >= 2)) || fail "--filter requires a value"
      bench_filter="$2"
      shift 2
      ;;
    --output)
      (($# >= 2)) || fail "--output requires a value"
      output_directory="$2"
      shift 2
      ;;
    --repeat)
      (($# >= 2)) || fail "--repeat requires a value"
      repeat_count="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

require_docker
if [[ "$command_name" != "prepare" ]]; then
  native_platform="$(native_docker_platform)"
  platform="${platform:-$native_platform}"
  [[ "$platform" == "$native_platform" ]] \
    || fail "Valgrind requires Docker's native platform ($native_platform), not emulated $platform"
fi

case "$command_name" in
  build-image)
    docker build \
      --platform "$platform" \
      --tag "$image" \
      --file "$SCRIPT_DIR/Dockerfile" \
      "$SCRIPT_DIR"
    ;;

  prepare)
    [[ -n "$repo" ]] || fail "prepare requires --repo"
    repo="$(canonical_directory "$repo")"
    require_command pnpm
    pnpm --dir "$repo" run bench:prepare
    ;;

  measure)
    [[ -n "$repo" ]] || fail "measure requires --repo"
    [[ -n "$fixtures" ]] || fail "measure requires --fixtures"
    [[ -n "$bench_target" ]] || fail "measure requires --bench"
    [[ -n "$bench_filter" ]] || fail "measure requires --filter"
    [[ -n "$output_directory" ]] || fail "measure requires --output"
    [[ "$bench_target" == "benches" || "$bench_target" == "rspack_sources" ]] \
      || fail "--bench must be benches or rspack_sources"
    [[ "$repeat_count" =~ ^[1-9][0-9]*$ ]] || fail "--repeat must be a positive integer"

    repo="$(canonical_directory "$repo")"
    fixtures="$(canonical_directory "$fixtures")"
    output_directory="$(mkdir -p "$output_directory" && canonical_directory "$output_directory")"
    [[ -z "$(find "$output_directory" -mindepth 1 -maxdepth 1 -print -quit)" ]] \
      || fail "output directory is not empty: $output_directory"
    [[ -f "$repo/Cargo.toml" && -f "$repo/rust-toolchain.toml" ]] \
      || fail "--repo is not an Rspack checkout: $repo"
    [[ -f "$fixtures/package.json" ]] \
      || fail "fixture directory is not prepared: $fixtures"
    docker image inspect "$image" >/dev/null 2>&1 \
      || fail "Docker image is missing; run build-image first: $image"

    source_sha="$(git -C "$repo" rev-parse HEAD)"
    source_status="$(git -C "$repo" status --short)"
    [[ -z "$source_status" ]] \
      || fail "measure requires a committed, clean source snapshot: $repo"
    git_common_directory="$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)"
    git_common_directory="$(canonical_directory "$git_common_directory")"
    image_id="$(docker image inspect "$image" --format '{{.Id}}')"
    target_cache_key="$(hash_text "$repo|$image_id|$platform")"
    target_volume="rspack-perf-valgrind-target-${target_cache_key}"
    registry_volume="rspack-perf-valgrind-cargo-registry"
    git_volume="rspack-perf-valgrind-cargo-git"
    container_fixtures="/benchcases-source"
    fixture_mode="read-only"
    copy_fixtures="false"
    if [[ "$bench_target" == "benches" && "$bench_filter" == *persistent_cache* ]]; then
      container_fixtures="/benchcases"
      fixture_mode="writable-copy"
      copy_fixtures="true"
    fi
    glibc_tunables=""
    if [[ "$platform" == "linux/amd64" ]]; then
      glibc_tunables="glibc.cpu.hwcaps=-AVX512F,-AVX2,-AVX,-AVX_Fast_Unaligned_Load,-ERMS,-Prefer_ERMS,-SSE4_2,-SSSE3"
    fi

    {
      echo "source_sha=$source_sha"
      echo "source_status=${source_status:-clean}"
      echo "repo=$repo"
      echo "fixtures=$fixtures"
      echo "git_common_directory=$git_common_directory"
      echo "image=$image"
      echo "image_id=$image_id"
      echo "platform=$platform"
      echo "bench_target=$bench_target"
      echo "bench_filter=$bench_filter"
      echo "fixture_mode=$fixture_mode"
      echo "repeat=$repeat_count"
      echo "target_volume=$target_volume"
      echo "measurement=callgrind-instructions"
      echo "CARGO_INCREMENTAL=0"
      echo "RUSTFLAGS=--cfg codspeed"
      echo "VALGRIND_FAIR_SCHED=yes"
      echo "MIMALLOC_PURGE_DELAY=-1"
      echo "GLIBC_TUNABLES=$glibc_tunables"
    } >"$output_directory/environment.txt"

    docker run \
      --rm \
      --platform "$platform" \
      --cap-add SYS_PTRACE \
      --security-opt seccomp=unconfined \
      --volume "$repo:/rspack:ro" \
      --volume "$git_common_directory:$git_common_directory:ro" \
      --volume "$fixtures:/benchcases-source:ro" \
      --volume "$output_directory:/results" \
      --volume "$target_volume:/target" \
      --volume "$registry_volume:/usr/local/cargo/registry" \
      --volume "$git_volume:/usr/local/cargo/git" \
      --workdir /rspack \
      --env CARGO_INCREMENTAL=0 \
      --env CARGO_TARGET_DIR=/target \
      --env "GLIBC_TUNABLES=$glibc_tunables" \
      --env MIMALLOC_PURGE_DELAY=-1 \
      --env "RSPACK_BENCHCASES_DIR=$container_fixtures" \
      "$image" \
      bash -c '
        set -euo pipefail
        trap "chmod -R a+rwX /results || true" EXIT

        bench_target="$1"
        bench_filter="$2"
        repeat_count="$3"
        copy_fixtures="$4"

        {
          rustc --version
          cargo --version
          valgrind --version
        } | tee /results/tool-versions.txt

        if ! RUSTFLAGS="--cfg codspeed" cargo build \
          --profile codspeed \
          -p rspack_benchmark \
          --bench "$bench_target" \
          --message-format=json-render-diagnostics \
          > /results/build.jsonl; then
          jq -r \
            "select(.reason == \"compiler-message\") | .message.rendered // empty" \
            /results/build.jsonl >&2
          exit 1
        fi

        benchmark_binary="$(jq -r \
          --arg target "$bench_target" \
          "select( \
            .reason == \"compiler-artifact\" \
            and .target.name == \$target \
            and (.target.kind | index(\"bench\")) \
          ) | .executable // empty" \
          /results/build.jsonl \
          | tail -n 1)"
        [[ -x "$benchmark_binary" ]] || {
          echo "benchmark binary not found: $benchmark_binary" >&2
          exit 1
        }

        if [[ "$copy_fixtures" == "true" ]]; then
          mkdir -p /benchcases
          cp -a /benchcases-source/. /benchcases/
        fi

        for ((run_index = 1; run_index <= repeat_count; run_index++)); do
          run_directory="/results/run-${run_index}"
          mkdir -p "$run_directory/tmp"
          TMPDIR="$run_directory/tmp" \
          BENCH_MODE=simulation \
          CODSPEED_CARGO_WORKSPACE_ROOT=/rspack \
          valgrind \
            --tool=callgrind \
            --instr-atstart=no \
            --fair-sched=yes \
            --error-exitcode=86 \
            --callgrind-out-file="$run_directory/callgrind.%p.out" \
            "$benchmark_binary" \
            "$bench_filter" \
            2>&1 | tee "$run_directory.log"

          mapfile -t measured_stages \
            < <(grep -F "Measured: " "$run_directory.log" || true)
          ((${#measured_stages[@]} == 1)) || {
            echo "benchmark filter must measure exactly one stage; got ${#measured_stages[@]}" >&2
            exit 1
          }
          [[ "${measured_stages[0]}" == *"$bench_filter"* ]] || {
            echo "measured stage does not match filter: ${measured_stages[0]}" >&2
            exit 1
          }
          measured_stage="${measured_stages[0]#Measured: }"

          shopt -s nullglob
          profile_files=("$run_directory"/callgrind.*.out*)
          ((${#profile_files[@]} > 0)) || {
            echo "no Callgrind profile produced for run $run_index" >&2
            exit 1
          }
          stage_profiles=()
          for profile_file in "${profile_files[@]}"; do
            if grep -Fqx \
              "desc: Trigger: Client Request: $measured_stage" \
              "$profile_file"; then
              stage_profiles+=("$profile_file")
            fi
          done
          ((${#stage_profiles[@]} == 1)) || {
            echo "expected one Callgrind profile for $measured_stage; got ${#stage_profiles[@]}" >&2
            exit 1
          }
          instruction_count=0
          while read -r totals_label totals_value; do
            [[ "$totals_label" == "totals:" ]] || continue
            [[ "$totals_value" =~ ^[0-9]+$ ]] || continue
            instruction_count=$((instruction_count + totals_value))
          done < <(grep -h "^totals:" "${stage_profiles[0]}" || true)
          [[ "$instruction_count" =~ ^[1-9][0-9]*$ ]] || {
            echo "invalid instruction count for run $run_index: $instruction_count" >&2
            exit 1
          }
          echo "instruction_count=$instruction_count" \
            | tee "$run_directory.instructions"
        done
      ' bash "$bench_target" "$bench_filter" "$repeat_count" "$copy_fixtures"

    echo "measurement results: $output_directory"
    ;;

  *)
    usage
    fail "unknown command: $command_name"
    ;;
esac
