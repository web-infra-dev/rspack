#!/usr/bin/env python3
"""Prepare the pinned benchmark inputs under the ignored target directory."""
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / 'target/goexec-compare/fixtures'
REVISION = 'a24cc8316e37328c13c153cfc896210b0ccf46a9'


def run(args):
    subprocess.run(args, cwd=DEST, check=True)


def main():
    if not DEST.exists():
        DEST.mkdir(parents=True)
        run(['git', 'init', '-q'])
        run(['git', 'remote', 'add', 'origin', 'https://github.com/rstackjs/rspack-benchcases.git'])
        run(['git', 'fetch', '--depth', '1', 'origin', REVISION])
        run(['git', 'checkout', '--detach', 'FETCH_HEAD'])
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=DEST, text=True).strip()
    if revision != REVISION:
        raise RuntimeError(f'Expected fixture revision {REVISION}, found {revision}')
    run(['pnpm', 'install', '--frozen-lockfile', '--ignore-scripts'])
    scaled = DEST / 'threejs-10x'
    (scaled / 'src').mkdir(parents=True, exist_ok=True)
    for i in range(10):
        shutil.copytree(DEST / 'threejs/src', scaled / f'src/threejs-{i}', dirs_exist_ok=True)
    imports = '\n'.join(f"import * as Three{i} from './threejs-{i}/Three.js';" for i in range(10))
    names = ', '.join(f'Three{i}' for i in range(10))
    (scaled / 'src/index.js').write_text(imports + '\nglobalThis.__rspackThreejs10x = [' + names + '];\n')
    (scaled / 'package.json').write_text('{"name":"threejs-10x"}\n')
    print(DEST)


if __name__ == '__main__':
    main()
