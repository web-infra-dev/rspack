import { spawn } from 'node:child_process';
import { access, cp, mkdir, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const BENCH_DIR = path.resolve(__dirname, '../../.bench');
const RSPACK_BENCH_CASES = path.join(BENCH_DIR, 'rspack-benchcases');
const THREEJS_SCALE = 10;
const THREEJS_PROJECT = 'threejs';
const THREEJS_SCALED_PROJECT = `${THREEJS_PROJECT}-${THREEJS_SCALE}x`;

async function pathExists(target) {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}

function run(command, args, options = {}) {
  console.log(`$ ${command} ${args.join(' ')}`);

  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      stdio: 'inherit',
    });

    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${command} exited with code ${code ?? 'unknown'}`));
    });
  });
}

async function rspackBenchcases() {
  await mkdir(BENCH_DIR, { recursive: true });

  if (await pathExists(RSPACK_BENCH_CASES)) {
    console.log('rspack-benchcases already exists, skipping');
    return;
  }

  await run('git', [
    'clone',
    '--depth=1',
    'https://github.com/rstackjs/rspack-benchcases.git',
    RSPACK_BENCH_CASES,
  ]);

  await Promise.all(
    ['.git', '.github'].map((item) =>
      rm(path.join(RSPACK_BENCH_CASES, item), {
        force: true,
        recursive: true,
      }),
    ),
  );

  await run('pnpm', ['install', '--ignore-scripts'], {
    cwd: RSPACK_BENCH_CASES,
  });
}

async function scaledThreejsBenchcase() {
  console.log(`preparing ${THREEJS_SCALED_PROJECT} benchmark case`);

  const sourceProject = path.join(RSPACK_BENCH_CASES, THREEJS_PROJECT);
  const sourceDir = path.join(sourceProject, 'src');

  if (!(await pathExists(sourceDir))) {
    throw new Error(
      `threejs benchmark source directory not found: ${sourceDir}`,
    );
  }

  const targetProject = path.join(RSPACK_BENCH_CASES, THREEJS_SCALED_PROJECT);
  const targetSrcDir = path.join(targetProject, 'src');

  await rm(targetProject, { force: true, recursive: true });
  await mkdir(targetSrcDir, { recursive: true });

  const namespaceNames = [];
  const entryImports = [];
  for (let i = 0; i < THREEJS_SCALE; i++) {
    const namespaceName = `Three${i}`;
    const copyName = `${THREEJS_PROJECT}-${i}`;
    namespaceNames.push(namespaceName);
    entryImports.push(
      `import * as ${namespaceName} from './${copyName}/Three.js';`,
    );
    await cp(sourceDir, path.join(targetSrcDir, copyName), { recursive: true });
  }

  await writeFile(
    path.join(targetSrcDir, 'index.js'),
    `${entryImports.join('\n')}\n\nglobalThis.__rspackThreejs10x = [${namespaceNames.join(', ')}];\n`,
  );
  await writeFile(
    path.join(targetProject, 'rspack.config.js'),
    `/** @type {import("@rspack/cli").Configuration} */\nmodule.exports = {\n\tentry: { main: "./src/index.js" }\n};\n`,
  );
  await writeFile(
    path.join(targetProject, 'package.json'),
    `${JSON.stringify({ name: THREEJS_SCALED_PROJECT }, null, 2)}\n`,
  );
}

await rspackBenchcases();
await scaledThreejsBenchcase();
