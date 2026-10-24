import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib


MAIN = r'''
use std::{fs, path::Path};
use rspack_resolver::{AliasValue, InternedPath, ResolveOptions, Resolver};

fn main() {
    assert!(cfg!(windows), "Run this probe on Windows");
    let args: Vec<_> = std::env::args().collect();
    let mode = &args[1];
    let root = Path::new(&args[2]).join(mode);
    fs::create_dir_all(&root).unwrap();
    let root = dunce::canonicalize(root).unwrap();
    let file = root.join("foo.js");
    let replacement = root.join("replacement.js");
    fs::write(&file, "export default 'original';").unwrap();
    fs::write(&replacement, "export default 'replacement';").unwrap();
    let mut directory = root.to_str().unwrap().to_owned();
    let mut expected = file.to_str().unwrap().to_owned();
    let mut held = Vec::new();
    let mut options = ResolveOptions::default();

    match mode.as_str() {
        "control" => {}
        "alias-slash-first" | "alias-native-first" => {
            let spelling = if mode == "alias-slash-first" {
                file.to_str().unwrap().replace('\\', "/")
            } else {
                file.to_str().unwrap().to_owned()
            };
            held.push(InternedPath::from(Path::new(&spelling)));
            options.symlinks = false;
            options.alias = vec![(
                format!("{}$", file.display()),
                vec![AliasValue::Path(replacement.to_str().unwrap().to_owned())],
            )];
            expected = replacement.to_str().unwrap().to_owned();
        }
        "canonicalized-mixed-context" => {
            let separator = directory.rfind('\\').unwrap();
            directory.replace_range(separator..separator + 1, "/");
            held.push(InternedPath::from(Path::new(&directory)));
            let spelling = file.to_str().unwrap().replace('\\', "/");
            held.push(InternedPath::from(Path::new(&spelling)));
        }
        _ => panic!("Unknown probe case"),
    }

    let resolver = Resolver::new(options);
    let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
    let resolved = runtime.block_on(resolver.resolve(Path::new(&directory), "./foo.js")).unwrap();
    let actual = resolved.path().to_str().unwrap();
    std::hint::black_box(&held);
    println!("case={mode}\nactual={actual:?}\nexpected={expected:?}");
    assert_eq!(actual, expected);
}
'''


def replace_once(source, old, new):
    assert source.count(old) == 1, old
    return source.replace(old, new)


parser = argparse.ArgumentParser()
parser.add_argument('--variant', choices=['reference', 'current', 'path-only', 'both-owned'], required=True)
parser.add_argument('--project', type=Path, required=True)
parser.add_argument('--prepare-only', action='store_true')
args = parser.parse_args()
repo = Path.cwd()
project = args.project.resolve()
refs = {'reference': 'b864274796', 'current': '6d64cea768', 'path-only': 'e8e5654b4d', 'both-owned': 'e8e5654b4d'}
source = subprocess.check_output(['git', 'show', f'{refs[args.variant]}:crates/rspack_resolver/src/cache.rs'], text=True)
if args.variant == 'path-only':
    source = replace_once(source, '#[cfg(unix)]\nuse rspack_paths::AssertUtf8;', 'use rspack_paths::AssertUtf8;')
    source = replace_once(source, '#[cfg(not(unix))]\ntype RealPathStorage = Utf8PathBuf;', '#[cfg(not(unix))]\ntype RealPathStorage = InternedPath;')
    source = replace_once(source, 'fn real_path_to_buf(path: &RealPathStorage) -> Utf8PathBuf {\n  path.clone()\n}', 'fn real_path_to_buf(path: &RealPathStorage) -> Utf8PathBuf {\n  path.as_path().assert_utf8().to_path_buf()\n}')

(repo / 'crates/rspack_resolver/src/cache.rs').write_text(source, newline='\n')
(project / 'src').mkdir(parents=True, exist_ok=True)
(project / 'cache.rs').write_text(source, newline='\n')
(project / 'src/main.rs').write_text(MAIN, newline='\n')
dependencies = tomllib.loads((repo / 'Cargo.toml').read_text())['workspace']['dependencies']
version = lambda name: dependencies[name] if isinstance(dependencies[name], str) else dependencies[name]['version']
manifest = f'''[package]
name = "resolver-windows-alias-probe"
version = "0.0.0"
edition = "2021"

[workspace]

[dependencies]
rspack_resolver = {{ path = {json.dumps((repo / 'crates/rspack_resolver').as_posix())}, default-features = false }}
tokio = {{ version = {json.dumps(version('tokio'))}, default-features = false, features = ["rt"] }}
dunce = {json.dumps(version('dunce'))}
'''
(project / 'Cargo.toml').write_text(manifest)
shutil.copyfile(repo / 'Cargo.lock', project / 'Cargo.lock')
if args.prepare_only:
    print(project / 'Cargo.toml')
    sys.exit(0)

target = repo / 'target/resolver-windows-alias-probe'
subprocess.run(['cargo', 'build', '--manifest-path', str(project / 'Cargo.toml'), '--target-dir', str(target)], check=True)
binary = target / 'debug/resolver-windows-alias-probe.exe'
results = []
for case in ['control', 'alias-slash-first', 'alias-native-first', 'canonicalized-mixed-context']:
    completed = subprocess.run([str(binary), case, str(project / 'fixtures')], text=True, capture_output=True)
    result = {'case': case, 'passed': completed.returncode == 0, 'exit_code': completed.returncode, 'stdout': completed.stdout, 'stderr': completed.stderr}
    results.append(result)
    print(json.dumps(result, indent=2), flush=True)

report = {'variant': args.variant, 'cache_sha256': hashlib.sha256(source.encode()).hexdigest(), 'cases': results}
(project / 'results.json').write_text(json.dumps(report, indent=2))
summary = os.environ.get('GITHUB_STEP_SUMMARY')
if summary:
    with open(summary, 'a') as output:
        output.write(f'## {args.variant}\n\n| Case | Result |\n| --- | --- |\n')
        for result in results:
            output.write(f"| {result['case']} | {'PASS' if result['passed'] else 'FAIL'} |\n")
controls_pass = all(result['passed'] for result in results if result['case'] in ['control', 'alias-native-first'])
reference_pass = args.variant != 'reference' or all(result['passed'] for result in results)
sys.exit(0 if controls_pass and reference_pass else 1)
