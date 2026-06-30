import { spawn } from 'node:child_process';
import { access, cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const BENCH_DIR = path.resolve(__dirname, '../../.bench');
const RSPACK_BENCH_CASES = path.join(BENCH_DIR, 'rspack-benchcases');
const THREEJS_SCALE = 10;
const THREEJS_PROJECT = 'threejs';
const THREEJS_SCALED_PROJECT = `${THREEJS_PROJECT}-${THREEJS_SCALE}x`;
const MISC_PROJECT = 'misc';
const MISC_SOURCE_MARKER = '.rspack-benchmark-source.json';
const MISC_ENTRY = 'src/index.ts';
const MISC_PROBLEMATIC_LIBRARIES = {
  '@atlaskit/editor-core': '120.1.0',
  '@atlaskit/media-core': '31.1.0',
  '@atlaskit/smart-card': '13.0.0',
  '@babel/runtime': '7.12.13',
  '@material-ui/core': '4.11.3',
  '@material-ui/icons': '4.11.2',
  '@material-ui/lab': '4.0.0-alpha.57',
  lodash: '4.17.20',
  'lodash-es': '4.17.20',
  react: '17.0.1',
  'react-dom': '17.0.1',
  'react-intl': '2.6.0',
  uuid: '8.3.2',
};

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
      env: {
        ...process.env,
        ...options.env,
      },
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

  await run('pnpm', ['install'], { cwd: RSPACK_BENCH_CASES });
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

async function miscBenchcase() {
  console.log(`preparing ${MISC_PROJECT} benchmark case`);

  const targetProject = path.join(RSPACK_BENCH_CASES, MISC_PROJECT);
  const sourceMarker = path.join(targetProject, MISC_SOURCE_MARKER);

  if (await pathExists(sourceMarker)) {
    console.log(`${MISC_PROJECT} benchmark case already exists, skipping`);
    return;
  }

  await rm(targetProject, { force: true, recursive: true });
  await mkdir(path.join(targetProject, 'src'), { recursive: true });

  const packageJsonPath = path.join(targetProject, 'package.json');
  await writeFile(
    packageJsonPath,
    `${JSON.stringify(
      {
        name: MISC_PROJECT,
        private: true,
        dependencies: MISC_PROBLEMATIC_LIBRARIES,
      },
      null,
      2,
    )}\n`,
  );

  const workspacePath = path.join(RSPACK_BENCH_CASES, 'pnpm-workspace.yaml');
  const workspace = await readFile(workspacePath, 'utf-8');
  if (!workspace.includes(`- "${MISC_PROJECT}"`)) {
    await writeFile(
      workspacePath,
      `${workspace.trimEnd()}\n  - "${MISC_PROJECT}"\n`,
    );
  }

  const entryPath = path.join(targetProject, MISC_ENTRY);
  await writeFile(
    entryPath,
    `import * as atlaskitEditorCore from '@atlaskit/editor-core';
import * as atlaskitMediaCore from '@atlaskit/media-core';
import * as atlaskitSmartCard from '@atlaskit/smart-card';
import * as materialCore from '@material-ui/core';
import * as materialLab from '@material-ui/lab';
import * as materialIcons from '@material-ui/icons';
import * as lodashEs from 'lodash-es';
import * as reactIntl from 'react-intl';
import * as uuid from 'uuid';
import * as react from 'react';
import * as reactDom from 'react-dom';

import '@babel/runtime/helpers/esm/extends';
import '@babel/runtime/helpers/esm/objectWithoutProperties';
import '@babel/runtime/helpers/esm/slicedToArray';
import '@babel/runtime/helpers/esm/defineProperty';
import '@babel/runtime/helpers/esm/typeof';

globalThis.__rspackProblematicLibraries = [
  atlaskitEditorCore,
  atlaskitMediaCore,
  atlaskitSmartCard,
  materialCore,
  materialLab,
  materialIcons,
  lodashEs,
  reactIntl,
  uuid,
  react,
  reactDom,
];

Promise.all([
  import('@atlaskit/editor-core'),
  import('@atlaskit/media-core'),
  import('@atlaskit/smart-card'),
  import('@material-ui/core'),
  import('@material-ui/lab'),
  import('@material-ui/icons'),
  import('@babel/runtime/helpers/esm/extends'),
  import('@babel/runtime/helpers/esm/objectWithoutProperties'),
  import('lodash-es'),
  import('react'),
  import('react-dom'),
  import('react-intl'),
  import('uuid'),
]).then((modules) => {
  globalThis.__rspackProblematicLibrariesAsync = modules;
});
`,
  );

  await run(
    'pnpm',
    [
      'install',
      '--prod',
      '--ignore-scripts',
      '--no-frozen-lockfile',
      '--config.confirmModulesPurge=false',
    ],
    {
      cwd: RSPACK_BENCH_CASES,
      env: { CI: 'true' },
    },
  );

  await writeFile(
    sourceMarker,
    `${JSON.stringify(
      {
        entry: MISC_ENTRY,
        dependencies: MISC_PROBLEMATIC_LIBRARIES,
        reason:
          'Covers module-concatenation root attempts that fail on shared ESM/CJS dependencies.',
      },
      null,
      2,
    )}\n`,
  );
}

await rspackBenchcases();
await scaledThreejsBenchcase();
await miscBenchcase();
