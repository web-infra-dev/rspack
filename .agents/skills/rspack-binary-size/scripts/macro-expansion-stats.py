#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import queue
import re
import subprocess
import threading
import time
from pathlib import Path
from urllib.parse import quote


SKIP_DIRS = {".git", "target", "node_modules", ".turbo", ".next", "dist", "coverage"}

BUILTIN_ATTRS = {
    "allow",
    "cfg",
    "cfg_attr",
    "clippy",
    "cold",
    "deny",
    "deprecated",
    "doc",
    "expect",
    "forbid",
    "inline",
    "link",
    "must_use",
    "no_mangle",
    "non_exhaustive",
    "path",
    "repr",
    "rustfmt",
    "should_panic",
    "test",
    "warn",
}

ATTR_RE = re.compile(r"#\s*!\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)|#\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)")
CALL_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_:]*)!\s*[\(\{\[]")
DERIVE_RE = re.compile(r"#\s*\[\s*derive\s*\((.*?)\)\s*\]", re.S)
IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_:]*")


def uri_for(path: Path) -> str:
    return "file://" + quote(str(path.resolve()))


def normalize_macro_name(name: str) -> str:
    return name.split("::")[-1]


def iter_rust_files(root: Path, scan_roots: list[str], max_bytes: int):
    for scan_root in scan_roots:
        base = root / scan_root
        if not base.exists():
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for filename in filenames:
                if not filename.endswith(".rs"):
                    continue
                path = Path(dirpath) / filename
                try:
                    if path.stat().st_size > max_bytes:
                        continue
                except OSError:
                    continue
                yield path


def line_col(text: str, offset: int) -> tuple[int, int]:
    line = text.count("\n", 0, offset)
    prev = text.rfind("\n", 0, offset)
    col = offset if prev < 0 else offset - prev - 1
    return line, col


def line_at(text: str, line: int) -> str:
    lines = text.splitlines()
    return lines[line] if 0 <= line < len(lines) else ""


def collect_candidates(root: Path, scan_roots: list[str], max_bytes: int):
    candidates = []
    per_macro: dict[tuple[str, str], dict[str, object]] = {}

    def add(kind: str, name: str, path: Path, line: int, col: int, snippet: str):
        norm = normalize_macro_name(name)
        key = (kind, norm)
        rel = str(path.relative_to(root))
        entry = per_macro.setdefault(key, {"count": 0, "files": set()})
        entry["count"] = int(entry["count"]) + 1
        entry["files"].add(rel)
        candidates.append(
            {
                "kind": kind,
                "name": norm,
                "raw_name": name,
                "path": path,
                "rel": rel,
                "line": line,
                "character": col,
                "snippet": snippet.strip()[:240],
            }
        )

    for path in iter_rust_files(root, scan_roots, max_bytes):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue

        for match in DERIVE_RE.finditer(text):
            body = match.group(1)
            for item in IDENT_RE.finditer(body):
                name = item.group(0)
                line, col = line_col(text, match.start(1) + item.start())
                add("derive", name, path, line, col, match.group(0).replace("\n", " "))

        for match in ATTR_RE.finditer(text):
            name = match.group(1) or match.group(2)
            if not name:
                continue
            norm = normalize_macro_name(name)
            if norm in BUILTIN_ATTRS:
                continue
            line, col = line_col(text, match.start())
            add("attribute", name, path, line, col, line_at(text, line))

        for match in CALL_RE.finditer(text):
            name = match.group(1)
            line, col = line_col(text, match.start(1))
            add("function_like", name, path, line, col, line_at(text, line))

    return candidates, per_macro


class RustAnalyzerClient:
    def __init__(self, root: Path, analyzer: str):
        self.root = root
        self.proc = subprocess.Popen(
            [analyzer],
            cwd=str(root),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,
        )
        self.next_id = 1
        self.response_queue: queue.Queue[object] = queue.Queue()
        self.notifications = []
        self.errors = []
        self.reader = threading.Thread(target=self._read_loop, daemon=True)
        self.reader.start()

    def _read_loop(self):
        try:
            while True:
                headers = {}
                while True:
                    line = self.proc.stdout.readline()
                    if not line:
                        return
                    if line in (b"\r\n", b"\n"):
                        break
                    key, value = line.decode("ascii").split(":", 1)
                    headers[key.lower()] = value.strip()
                length = int(headers.get("content-length", "0"))
                if length <= 0:
                    continue
                payload = self.proc.stdout.read(length)
                self.response_queue.put(json.loads(payload.decode("utf-8")))
        except Exception as error:
            self.errors.append(f"reader failed: {error}")

    def send(self, message: dict[str, object]):
        encoded = json.dumps(message, separators=(",", ":")).encode("utf-8")
        header = f"Content-Length: {len(encoded)}\r\n\r\n".encode("ascii")
        assert self.proc.stdin is not None
        self.proc.stdin.write(header + encoded)
        self.proc.stdin.flush()

    def request(self, method: str, params: object, timeout: float = 60.0):
        request_id = self.next_id
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params})
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                message = self.response_queue.get(timeout=0.25)
            except queue.Empty:
                if self.proc.poll() is not None:
                    raise RuntimeError(f"rust-analyzer exited with code {self.proc.returncode}")
                continue
            if "id" in message and message["id"] == request_id:
                if "error" in message:
                    raise RuntimeError(json.dumps(message["error"], ensure_ascii=False))
                return message.get("result")
            self.notifications.append(message)
        raise TimeoutError(f"timed out waiting for {method}")

    def notify(self, method: str, params: object):
        self.send({"jsonrpc": "2.0", "method": method, "params": params})

    def initialize(self):
        result = self.request(
            "initialize",
            {
                "processId": os.getpid(),
                "rootUri": uri_for(self.root),
                "capabilities": {},
                "workspaceFolders": [{"uri": uri_for(self.root), "name": self.root.name}],
            },
            timeout=120.0,
        )
        self.notify("initialized", {})
        return result

    def shutdown(self):
        try:
            self.request("shutdown", None, timeout=10.0)
            self.notify("exit", None)
        except Exception as error:
            self.errors.append(f"shutdown failed: {error}")
        finally:
            try:
                self.proc.terminate()
            except Exception:
                pass


def write_tsv(path: Path, rows: list[list[object]], header: list[str]):
    with path.open("w", encoding="utf-8") as output:
        output.write("\t".join(header) + "\n")
        for row in rows:
            output.write("\t".join(str(value).replace("\t", " ") for value in row) + "\n")


def write_scan_reports(out: Path, candidates, per_macro):
    summary_rows = []
    for (kind, name), data in sorted(
        per_macro.items(),
        key=lambda item: (-int(item[1]["count"]), item[0][0], item[0][1]),
    ):
        summary_rows.append([kind, name, data["count"], len(data["files"])])
    write_tsv(out / "macro-occurrences.tsv", summary_rows, ["kind", "name", "count", "files"])

    location_rows = [
        [item["kind"], item["name"], item["rel"], item["line"] + 1, item["character"] + 1, item["snippet"]]
        for item in candidates
    ]
    write_tsv(out / "macro-locations.tsv", location_rows, ["kind", "name", "file", "line", "character", "snippet"])

    per_file: dict[str, int] = {}
    for item in candidates:
        per_file[item["rel"]] = per_file.get(item["rel"], 0) + 1
    file_rows = [[count, rel] for rel, count in sorted(per_file.items(), key=lambda item: (-item[1], item[0]))]
    write_tsv(out / "macro-files.tsv", file_rows, ["count", "file"])


def expand_with_rust_analyzer(root: Path, out: Path, candidates):
    analyzer = os.environ.get("RUST_ANALYZER", "rust-analyzer")
    limit = int(os.environ.get("RA_EXPAND_LIMIT", "200"))
    pattern = os.environ.get("RA_EXPAND_FILTER", "")
    name_filter = re.compile(pattern) if pattern else None
    timeout = float(os.environ.get("RA_EXPAND_TIMEOUT", "45"))

    selected = []
    for item in candidates:
        if name_filter and not name_filter.search(item["name"]):
            continue
        selected.append(item)
        if len(selected) >= limit:
            break

    expansions_dir = out / "rust-analyzer-expansions"
    expansions_dir.mkdir(exist_ok=True)
    rows = []
    errors = []
    opened = set()
    client = RustAnalyzerClient(root, analyzer)
    try:
        client.initialize()
        for item in selected:
            path = item["path"]
            uri = uri_for(path)
            if uri not in opened:
                client.notify(
                    "textDocument/didOpen",
                    {
                        "textDocument": {
                            "uri": uri,
                            "languageId": "rust",
                            "version": 1,
                            "text": path.read_text(encoding="utf-8", errors="replace"),
                        }
                    },
                )
                opened.add(uri)

            params = {
                "textDocument": {"uri": uri},
                "position": {"line": item["line"], "character": item["character"]},
            }
            try:
                result = client.request("rust-analyzer/expandMacro", params, timeout=timeout)
            except Exception as error:
                errors.append(f"{item['rel']}:{item['line'] + 1}:{item['character'] + 1} {item['name']}: {error}")
                rows.append(["error", item["kind"], item["name"], item["rel"], item["line"] + 1, 0, 0, ""])
                continue

            if not result:
                rows.append(["empty", item["kind"], item["name"], item["rel"], item["line"] + 1, 0, 0, ""])
                continue

            expansion = result.get("expansion", "") if isinstance(result, dict) else str(result)
            macro_name = result.get("name", item["name"]) if isinstance(result, dict) else item["name"]
            digest = hashlib.sha256(expansion.encode("utf-8")).hexdigest()[:16]
            filename = f"{len(rows):05d}-{item['name']}-{digest}.rs".replace("/", "_")
            (expansions_dir / filename).write_text(expansion, encoding="utf-8")
            rows.append(
                [
                    "ok",
                    item["kind"],
                    macro_name,
                    item["rel"],
                    item["line"] + 1,
                    len(expansion.encode("utf-8")),
                    expansion.count("\n") + 1 if expansion else 0,
                    filename,
                ]
            )
    finally:
        client.shutdown()

    write_tsv(
        out / "rust-analyzer-expansions.tsv",
        rows,
        ["status", "kind", "name", "file", "line", "bytes", "lines", "expansion_file"],
    )
    if errors or client.errors:
        (out / "rust-analyzer-errors.txt").write_text("\n".join(errors + client.errors) + "\n", encoding="utf-8")


def cargo_expand(root: Path, out: Path):
    crates = os.environ.get("CARGO_EXPAND_CRATES", "rspack_binding_api").split()
    markers = [
        "__napi",
        "FromNapiValue",
        "ToNapiValue",
        "ValidateNapiValue",
        "TypeName",
        "CallbackInfo",
        "register_class",
        "register_module_export",
        "ThreadsafeFunction",
        "Either",
    ]
    rows = []
    for crate in crates:
        expanded = out / f"cargo-expand-{crate}.rs"
        stderr = out / f"cargo-expand-{crate}.stderr.txt"
        with expanded.open("w", encoding="utf-8") as stdout, stderr.open("w", encoding="utf-8") as err:
            proc = subprocess.run(["cargo", "expand", "-p", crate], cwd=root, stdout=stdout, stderr=err, check=False)
        if proc.returncode != 0:
            rows.append([crate, "error", 0, 0, ""])
            continue
        text = expanded.read_text(encoding="utf-8", errors="replace")
        marker_counts = ",".join(f"{marker}={text.count(marker)}" for marker in markers)
        rows.append([crate, "ok", len(text.encode("utf-8")), text.count("\n") + 1, marker_counts])
    write_tsv(out / "cargo-expand-summary.tsv", rows, ["crate", "status", "bytes", "lines", "marker_counts"])


def main():
    parser = argparse.ArgumentParser(description="Collect generic Rust macro usage and optional macro expansion sizes.")
    parser.add_argument("--root", default=".", help="repository root")
    parser.add_argument("--out", required=True, help="output directory")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)

    scan_roots = os.environ.get("MACRO_SCAN_ROOTS", "crates packages .agents").split()
    max_bytes = int(os.environ.get("MACRO_SCAN_MAX_BYTES", str(1024 * 1024)))

    candidates, per_macro = collect_candidates(root, scan_roots, max_bytes)
    write_scan_reports(out, candidates, per_macro)

    backend = os.environ.get("MACRO_EXPAND_BACKEND", "none")
    if backend == "rust-analyzer":
        try:
            expand_with_rust_analyzer(root, out, candidates)
        except FileNotFoundError:
            (out / "rust-analyzer.skipped.txt").write_text(
                "rust-analyzer was not found. Set RUST_ANALYZER=/path/to/rust-analyzer or use MACRO_EXPAND_BACKEND=none.\n",
                encoding="utf-8",
            )
        except Exception as error:
            (out / "rust-analyzer.failed.txt").write_text(f"{error}\n", encoding="utf-8")
    elif backend == "cargo-expand":
        try:
            cargo_expand(root, out)
        except FileNotFoundError as error:
            (out / "cargo-expand.skipped.txt").write_text(f"{error}\n", encoding="utf-8")
    elif backend != "none":
        (out / "backend.skipped.txt").write_text(f"unknown MACRO_EXPAND_BACKEND={backend}\n", encoding="utf-8")

    print(f"macro candidates: {len(candidates)}")


if __name__ == "__main__":
    main()
