#!/usr/bin/env python3
"""Temporary Linux linker experiment; this branch is not a product change."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import time

repo = Path(__file__).resolve().parents[2]
out = repo / '.linux-lld-results'
out.mkdir(exist_ok=True)


def save(name, value):
    (out / name).write_text(json.dumps(value, indent=2))


def output(args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def measured(args, name, **kwargs):
    start = time.perf_counter()
    with (out / (name + '.log')).open('w') as log:
        p = subprocess.Popen(list(map(str, args)), stdout=log, stderr=subprocess.STDOUT, **kwargs)
        _, status, usage = os.wait4(p.pid, 0)
        p.returncode = os.waitstatus_to_exitcode(status)
    row = {'seconds': time.perf_counter() - start, 'exit': p.returncode}
    row['peak_rss_kib'] = usage.ru_maxrss
    if p.returncode:
        print((out / (name + '.log')).read_text()[-16000:], flush=True)
        raise RuntimeError(f'{name}: exit {p.returncode}')
    return row


def rustc_wrapper():
    args = sys.argv[2:]
    if '--crate-name' in args and args[args.index('--crate-name') + 1] == 'rspack_node':
        linker = next((a.split('=', 1)[1] for a in args if a.startswith('linker=')), None)
        if linker is None:
            linker = next((a.split('=', 1)[1] for a in args if a.startswith('-Clinker=')), None)
        assert linker and linker.endswith('gcc'), linker
        os.environ['LLD_AB_REAL_LINKER'] = linker
        # Calling the capture driver "gcc" preserves rustc's original linker inference.
        args.extend(['-C', 'linker=' + str(out / 'capture/gcc')])
    os.execv(args[0], args)


def capture_link():
    mode = os.environ['LLD_AB_MODE']
    args = sys.argv[2:]
    dest = out / ('inputs-' + mode)
    dest.mkdir(exist_ok=True)
    # Preserve temporary version scripts and post-LTO objects before rustc deletes them.
    for arg in list(args):
        for token in re.findall(r'/[^,=\s]+', arg):
            p = Path(token)
            if p.is_file() and p.parent.name.lstrip('.').startswith('rustc'):
                target = dest / p.parent.name
                shutil.copytree(p.parent, target, dirs_exist_ok=True)
                args = [a.replace(str(p.parent), str(target)) for a in args]
    for i, arg in enumerate(args):
        p = Path(arg)
        if not arg.startswith('-') and p.is_file() and p.suffix in ('.o', '.rlib', '.a'):
            target = dest / (str(i) + '-' + p.name)
            shutil.copy2(p, target)
            args[i] = str(target)
    driver = os.environ['LLD_AB_REAL_LINKER']
    save('command-' + mode + '.json', {'driver': driver, 'cwd': os.getcwd(), 'args': args})
    cmd = [driver] + (['-B' + str(out / 'lld-bin')] if mode == 'lld' else []) + args
    start = time.perf_counter()
    p = subprocess.run(cmd)
    save('initial-link-' + mode + '.json', {'seconds': time.perf_counter() - start, 'exit': p.returncode})
    sys.exit(p.returncode)


def build_and_link():
    profile = os.environ.get('LLD_AB_PROFILE', 'release')
    for folder in ['capture', 'lld-bin', 'variants']:
        (out / folder).mkdir(exist_ok=True)
    script = shlex.quote(str(Path(__file__).resolve()))
    for name, mode in [('rustc-wrapper', 'rustc'), ('capture/gcc', 'link')]:
        p = out / name
        p.write_text(f'#!/bin/sh\nexec python3 {script} {mode} "$@"\n')
        p.chmod(0o755)
    libdir = Path(output(['rustc', '--print', 'target-libdir']))
    lld = libdir.parent / 'bin/rust-lld'
    assert lld.is_file(), lld
    p = out / 'lld-bin/ld'
    p.write_text('#!/bin/sh\nexec ' + shlex.quote(str(lld)) + ' -flavor gnu "$@"\n')
    p.chmod(0o755)
    save('environment.json', {
        'source': output(['git', 'rev-parse', 'HEAD']),
        'rustc': output(['rustc', '-vV']),
        'lld': output([str(lld), '-flavor', 'gnu', '--version']),
        'cpu': output(['lscpu']), 'memory': output(['free', '-m']),
        'node': output(['node', '--version']),
        'benchmark': output(['git', '-C', str(repo / '.linux-lld-btp'), 'rev-parse', 'HEAD']),
    })
    full = {}
    for mode in ['baseline', 'lld']:
        env = {**os.environ, 'RUST_TARGET': 'x86_64-unknown-linux-gnu', 'USE_NAPI_CROSS': '1', 'TARGET_CC': 'clang', 'RUSTC_WRAPPER': str(out / 'rustc-wrapper'), 'LLD_AB_MODE': mode, 'CARGO_TARGET_DIR': str(repo / ('.linux-lld-target-' + mode)), 'CARGO_INCREMENTAL': '0'}
        env.pop('RUSTC_WORKSPACE_WRAPPER', None)
        full[mode] = measured(['pnpm', 'build:binding:' + profile], 'build-' + mode, env=env, cwd=repo)
        save('full-build.json', full)
        print('FULL BUILD', mode, full[mode], flush=True)
    command = json.loads((out / 'command-baseline.json').read_text())
    driver = command['driver']
    save('driver.json', {'path': driver, 'version': output([driver, '--version']), 'sysroot': output([driver, '-print-sysroot']), 'default_ld': output([driver, '-print-prog-name=ld']), 'lld_override': output([driver, '-B' + str(out / 'lld-bin'), '-print-prog-name=ld'])})
    rows = []
    binary = out / 'relink.node'
    for iteration in range(14):
        for mode in (['baseline', 'lld'] if iteration % 2 == 0 else ['lld', 'baseline']):
            args = command['args'].copy()
            args[args.index('-o') + 1] = str(binary)
            cmd = [driver] + (['-B' + str(out / 'lld-bin')] if mode == 'lld' else []) + args
            row = measured(cmd, f'link-{mode}-{iteration}', cwd=command['cwd'])
            row.update(variant=mode, iteration=iteration, warmup=iteration < 2, bytes=binary.stat().st_size)
            rows.append(row)
            shutil.copy2(binary, out / 'variants' / (mode + '.node'))
            save('links.json', rows)
    for mode in ['baseline', 'lld']:
        binary = out / 'variants' / (mode + '.node')
        subprocess.run(['node', '-e', 'const b=require(process.argv[1]);if(!Object.keys(b).length)throw Error("empty binding");console.log(Object.keys(b).length)', str(binary)], check=True)
        for label, args in [('comments', ['readelf', '-p', '.comment']), ('versions', ['readelf', '--version-info']), ('dependencies', ['readelf', '-d'])]:
            (out / f'{mode}-{label}.log').write_text(output([*args, str(binary)]))
    assert 'LLD' in (out / 'lld-comments.log').read_text()
    print('LINK COMPARISON COMPLETE', flush=True)


def benchmark():
    btp = repo / '.linux-lld-btp'
    for name, source in [('core', 'packages/rspack'), ('cli', 'packages/rspack-cli')]:
        p = btp / 'node_modules/@rspack' / name
        if p.is_symlink(): p.unlink()
        elif p.exists(): shutil.rmtree(p)
        p.symlink_to(repo / source, target_is_directory=True)
    observer = out / 'observe.cjs'
    observer.write_text('const old=process.dlopen;process.dlopen=function(m,f,...a){console.error("NATIVE="+f);return old.call(this,m,f,...a)};')
    variants = {'baseline-a': 'baseline', 'baseline-b': 'baseline', 'lld': 'lld'}
    orders = list(itertools.permutations(variants))
    reference = {}
    rows = []
    for iteration in range(14):
        for case in ['react-dashboard', 'react-10k']:
            cwd = btp / 'cases' / case
            for label in orders[iteration % len(orders)]:
                mode = variants[label]
                native = str(out / 'variants' / (mode + '.node'))
                env = {**os.environ, 'NODE_ENV': 'production', 'NO_COLOR': '1', 'NAPI_RS_NATIVE_LIBRARY_PATH': native, 'NODE_DISABLE_COMPILE_CACHE': '1'}
                env.pop('NODE_OPTIONS', None)
                if iteration == 0: env['NODE_OPTIONS'] = '--require ' + str(observer)
                for base in [btp, cwd]: shutil.rmtree(base / 'node_modules/.cache', ignore_errors=True)
                for phase in ['cold', 'warm']:
                    shutil.rmtree(cwd / 'dist', ignore_errors=True)
                    name = f'bench-{case}-{label}-{iteration}-{phase}'
                    row = measured(['node', '--run', 'build:rspack'], name, cwd=cwd, env=env)
                    if iteration == 0: assert 'NATIVE=' + native in (out / (name + '.log')).read_text()
                    files = {str(f.relative_to(cwd / 'dist')): hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted((cwd / 'dist').rglob('*')) if f.is_file()}
                    assert files, case
                    if case not in reference: reference[case] = files
                    assert files == reference[case], f'output mismatch: {case} {label} {phase}'
                    row.update(case=case, variant=label, iteration=iteration, phase=phase, warmup=iteration < 2)
                    rows.append(row)
                    save('benchmarks.json', rows)
                    print(json.dumps(row), flush=True)
                    time.sleep(0.5)
    save('output-hashes.json', reference)


if __name__ == '__main__':
    {'rustc': rustc_wrapper, 'link': capture_link, 'build': build_and_link, 'bench': benchmark}[sys.argv[1]]()
