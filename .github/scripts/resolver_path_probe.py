import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


# Mirrors enhanced-resolve/lib/util/path.js. Node supplies both path dialects,
# regardless of the host OS.
ORACLE = r'''
const { posix, win32 } = require("node:path");
const type = p => {
  if (!p) return "empty";
  if (p[0] === "/") return "posix";
  if (/^(?:[a-z]:[\\/]|[a-z]:$|\\\\)/i.test(p)) return "windows";
  if (/^\.{1,2}(?:\/|$)/.test(p)) return "relative";
  return p[0] === "#" ? "internal" : "normal";
};
const normalize = p => {
  const t = type(p);
  if (t === "empty") return p;
  if (t === "windows") return win32.normalize(p);
  const result = posix.normalize(p);
  return t === "relative" && type(result) !== "relative" ? `./${result}` : result;
};
// Keep the resolver's existing POSIX contract: no trailing separator or
// leading './', and parent traversal stops at the start of the path.
const resolverPosix = p => {
  const segments = [];
  for (const part of p.split("/")) {
    if (part === "..") segments.pop();
    else if (part && part !== ".") segments.push(part);
  }
  return (p.startsWith("/") ? "/" : "") + segments.join("/");
};
const resolverNormalize = p => type(p) === "windows" ? normalize(p) : resolverPosix(p);
const join = (root, request) => {
  if (!request) return resolverNormalize(root);
  const t = type(request);
  if (t === "windows" || t === "posix") return resolverNormalize(request);
  const r = type(root);
  if (r === "windows") return win32.normalize(`${root}\\${request}`);
  return root ? resolverPosix(`${root}/${request}`) : resolverNormalize(request);
};
const paths = ["", ".", "..", "./a", "./a/../b", "a/../../b/", "/", "/a/../b/",
  "/a\\b/../c", "//server/share/a", "\\a\\b", "a\\b", "C:", "C://",
  "C:/a/../b", "C:\\a\\..\\b", "c:\\a/b//c/../", "C:relative",
  "\\\\server\\share", "\\\\server\\share\\a\\..\\b", "\\\\?\\C:\\a\\..\\b",
  "\\\\.\\C:\\a\\..\\b", "\\\\?\\UNC\\server\\share\\a\\..\\b",
  "#internal", "C:/中文/../文件.js", "C:/a/../../../b", "C:/a/././", "\\\\server",
  "\\\\?\\C:", "\\\\?\\C:\\a\\..\\..", "\\\\.\\PHYSICALDRIVE0", "\\\\?\\COM1:",
  "\\\\?\\COM1:\\a\\..", "\\\\.\\LPT9:", "\\\\?\\COM¹:\\a\\..", "\\\\?\\NUL:child"];
const roots = ["C:/project", "C:\\project", "c:\\project/sub", "C:",
  "\\\\server\\share\\project", "\\\\?\\C:\\project", "/project", "./project", "project", "", "#internal"];
const requests = ["", ".", "..", "./foo.js", "../foo.js", "a\\b/../c/", "C:/a/../b",
  "D:\\a\\..\\b", "/a/../b", "\\\\server\\share\\a", "C:", "中文/../文件.js"];
let seed = 42;
const random = n => { seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0; return seed % n; };
for (let i = 0; i < 1000; i++) {
  let p = roots[random(6)];
  for (let j = random(8); j >= 0; j--) p += ["/", "\\"][random(2)] + ["a", "..", ".", "", "文件", "b.js"][random(6)];
  paths.push(p);
}
const cases = paths.map(p => ({args: [p], expected: resolverNormalize(p)}));
for (const root of roots) for (const request of requests) cases.push({args: [root, request], expected: join(root, request)});
process.stdout.write(JSON.stringify(cases));
'''

MAIN = r'''
use std::io::{self, Read};
use camino::Utf8Path;
use path::PathUtil;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: serde_json::Value = serde_json::from_str(&input).unwrap();
    let results: Vec<_> = cases.as_array().unwrap().iter().map(|case| {
        let args = case["args"].as_array().unwrap();
        let base = Utf8Path::new(args[0].as_str().unwrap());
        let result = if args.len() == 1 { base.normalize() }
            else { base.normalize_with(args[1].as_str().unwrap()) };
        result.into_string()
    }).collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
'''

RESOLVER = r'''
use std::io;
use std::path::{Path, PathBuf};
use rspack_resolver::{AliasValue, FileMetadata, FileSystem, ResolveOptions, ResolverGeneric};

struct WindowsFileSystem;

#[async_trait::async_trait]
impl FileSystem for WindowsFileSystem {
    async fn read(&self, _: &Path) -> io::Result<Vec<u8>> {
        Err(io::ErrorKind::NotFound.into())
    }
    async fn read_to_string(&self, _: &Path) -> io::Result<String> {
        Err(io::ErrorKind::NotFound.into())
    }
    async fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        match path.to_str().unwrap() {
            r"C:\project\foo.js" | r"C:\project\replacement.js" => Ok(FileMetadata::new(true, false, false)),
            r"C:\" | r"C:\project" => Ok(FileMetadata::new(false, true, false)),
            _ => Err(io::ErrorKind::NotFound.into()),
        }
    }
    async fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        self.metadata(path).await
    }
    async fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(path.to_path_buf())
    }
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
    runtime.block_on(async {
        for alias in [false, true] {
            let mut options = ResolveOptions { symlinks: false, ..Default::default() };
            if alias {
                options.alias = vec![(r"C:\project\foo.js$".into(),
                    vec![AliasValue::Path(r"C:\project\replacement.js".into())])];
            }
            let resolver = ResolverGeneric::new_with_file_system(WindowsFileSystem, options);
            for (context, request) in [
                ("C:/project", "./foo.js"),
                (r"C:\project", "./foo.js"),
                (r"C:\project", "C:/project/sub/../foo.js"),
                ("C:/project", r"C:\project\sub\..\foo.js"),
            ] {
                let result = resolver.resolve(Path::new(context), request).await.unwrap();
                let expected = if alias { r"C:\project\replacement.js" } else { r"C:\project\foo.js" };
                assert_eq!(result.path().to_str().unwrap(), expected, "{context} + {request}");
            }
        }
    });
    println!("8 Windows-style absolute/relative/alias resolver cases passed");
}
'''


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--path-source", type=Path)
    parser.add_argument("--cases", type=Path)
    parser.add_argument("--resolver", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    source = (args.path_source or root / "crates/rspack_resolver/src/path.rs").resolve()
    cases = json.loads(args.cases.read_text(encoding="utf-8") if args.cases else
                       subprocess.check_output(["node", "-e", ORACLE], encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="resolver-path-probe-") as directory:
        project = Path(directory)
        (project / "src").mkdir()
        (project / "Cargo.toml").write_text(
            '[package]\nname = "resolver_path_probe"\nversion = "0.0.0"\nedition = "2021"\n'
            '[workspace]\n[dependencies]\ncamino = "1.2.1"\nserde_json = "1"\n'
            'tokio = { version = "1", features = ["macros", "rt"] }\n'
        )
        (project / "src/main.rs").write_text(
            f'#[allow(dead_code)]\n#[path = {json.dumps(str(source))}]\nmod path;\n' + MAIN
        )
        env = os.environ.copy()
        env.setdefault("CARGO_TARGET_DIR", str(root / "target/resolver-path-probe"))
        subprocess.run(["cargo", "build", "--manifest-path", str(project / "Cargo.toml")],
                       cwd=root, env=env, check=True)
        binary = Path(env["CARGO_TARGET_DIR"]) / "debug" / (
            "resolver_path_probe.exe" if os.name == "nt" else "resolver_path_probe"
        )
        results = json.loads(subprocess.check_output(
            [str(binary)], input=json.dumps(cases), encoding="utf-8"
        ))
        failures = [dict(case, actual=actual) for case, actual in zip(cases, results, strict=True)
                    if actual != case["expected"]]
        for failure in failures[:12]:
            print(json.dumps(failure))
        print(f"{len(cases) - len(failures)}/{len(cases)} cases match Windows normalization and POSIX compatibility rules")
        if not failures and args.resolver:
            with (project / "Cargo.toml").open("a") as manifest:
                manifest.write(
                    f'rspack_resolver = {{ path = {json.dumps(str(root / "crates/rspack_resolver"))}, default-features = false }}\n'
                    'async-trait = "0.1"\n'
                )
            (project / "src/main.rs").write_text(RESOLVER)
            subprocess.run(["cargo", "run", "--manifest-path", str(project / "Cargo.toml")],
                           cwd=root, env=env, check=True)
        return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
