import { spawn } from 'node:child_process';
import { access, mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const BENCH_DIR = path.resolve(__dirname, '../../.bench');
const RSPACK_BENCH_CASES = path.join(BENCH_DIR, 'rspack-benchcases');
const REQUIRED_BENCH_CASES = ['threejs-10x'];

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
    const missingBenchCases = [];
    for (const benchCase of REQUIRED_BENCH_CASES) {
      if (!(await pathExists(path.join(RSPACK_BENCH_CASES, benchCase)))) {
        missingBenchCases.push(benchCase);
      }
    }

    if (missingBenchCases.length === 0) {
      console.log('rspack-benchcases already exists, skipping');
      return;
    }

    console.log(
      `rspack-benchcases is missing ${missingBenchCases.join(', ')}, refreshing`,
    );
    await rm(RSPACK_BENCH_CASES, { force: true, recursive: true });
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

  for (const benchCase of REQUIRED_BENCH_CASES) {
    if (!(await pathExists(path.join(RSPACK_BENCH_CASES, benchCase)))) {
      throw new Error(
        `rspack-benchcases clone is missing required benchmark case: ${benchCase}`,
      );
    }
  }

  await run('pnpm', ['install'], { cwd: RSPACK_BENCH_CASES });
}

await rspackBenchcases();
