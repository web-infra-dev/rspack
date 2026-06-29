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
const ROME_PROJECT = 'rome-ts';
const ROME_REPOSITORY = 'https://github.com/rome/tools.git';
const ROME_COMMIT = 'd95a3a7aab90773c9b36d9c82a08c8c4c6b68aa5';
const ROME_SOURCE_MARKER = '.rspack-benchmark-source.json';
const ROME_PROBLEMATIC_LIBS_MARKER =
  '.rspack-problematic-libraries-source.json';
const ROME_PROBLEMATIC_LIBS_ENTRY = 'benchmark/problematic-libs-entry.ts';
const ROME_PROBLEMATIC_LIBRARIES = {
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

function isRomeBenchSource(source) {
  const normalized = source.split(path.sep).join('/');
  return (
    !normalized.includes('/test-fixtures/') &&
    !normalized.includes('/__snapshots__/') &&
    !normalized.endsWith('.test.ts') &&
    !normalized.endsWith('.test.tsx')
  );
}

async function romeTsBenchcase() {
  console.log(`preparing ${ROME_PROJECT} benchmark case`);

  const targetProject = path.join(RSPACK_BENCH_CASES, ROME_PROJECT);
  const sourceMarker = path.join(targetProject, ROME_SOURCE_MARKER);

  if (await pathExists(sourceMarker)) {
    console.log(`${ROME_PROJECT} benchmark case already exists, skipping`);
    return;
  }

  const checkoutDir = path.join(BENCH_DIR, '.rome-tools-checkout');
  await rm(checkoutDir, { force: true, recursive: true });
  await rm(targetProject, { force: true, recursive: true });

  await run('git', ['init', checkoutDir]);
  await run('git', ['fetch', '--depth=1', ROME_REPOSITORY, ROME_COMMIT], {
    cwd: checkoutDir,
  });
  await run('git', ['checkout', '--force', 'FETCH_HEAD'], {
    cwd: checkoutDir,
  });

  await mkdir(path.join(targetProject, 'packages'), { recursive: true });
  await Promise.all([
    cp(
      path.join(checkoutDir, 'packages', '@romejs'),
      path.join(targetProject, 'packages', '@romejs'),
      { filter: isRomeBenchSource, recursive: true },
    ),
    cp(
      path.join(checkoutDir, 'packages', '@romejs-runtime'),
      path.join(targetProject, 'packages', '@romejs-runtime'),
      { filter: isRomeBenchSource, recursive: true },
    ),
    cp(
      path.join(checkoutDir, 'packages', 'rome'),
      path.join(targetProject, 'packages', 'rome'),
      { filter: isRomeBenchSource, recursive: true },
    ),
    cp(
      path.join(checkoutDir, 'tsconfig.json'),
      path.join(targetProject, 'tsconfig.json'),
    ),
    cp(
      path.join(checkoutDir, 'package.json'),
      path.join(targetProject, 'package.json'),
    ),
  ]);

  await writeFile(
    sourceMarker,
    `${JSON.stringify(
      {
        repository: ROME_REPOSITORY,
        commit: ROME_COMMIT,
      },
      null,
      2,
    )}\n`,
  );

  await rm(checkoutDir, { force: true, recursive: true });
}

async function romeProblematicLibrariesBenchcase() {
  console.log(`preparing ${ROME_PROJECT} problematic libraries benchmark input`);

  const targetProject = path.join(RSPACK_BENCH_CASES, ROME_PROJECT);
  const sourceMarker = path.join(targetProject, ROME_SOURCE_MARKER);
  const problematicLibrariesMarker = path.join(
    targetProject,
    ROME_PROBLEMATIC_LIBS_MARKER,
  );

  if (!(await pathExists(sourceMarker))) {
    throw new Error(
      `${ROME_PROJECT} benchmark source marker not found: ${sourceMarker}`,
    );
  }

  if (await pathExists(problematicLibrariesMarker)) {
    console.log(
      `${ROME_PROJECT} problematic libraries input already exists, skipping`,
    );
    return;
  }

  const packageJsonPath = path.join(targetProject, 'package.json');
  const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf-8'));
  packageJson.dependencies = {
    ...(packageJson.dependencies ?? {}),
    ...ROME_PROBLEMATIC_LIBRARIES,
  };
  await writeFile(
    packageJsonPath,
    `${JSON.stringify(packageJson, null, 2)}\n`,
  );

  const workspacePath = path.join(RSPACK_BENCH_CASES, 'pnpm-workspace.yaml');
  const workspace = await readFile(workspacePath, 'utf-8');
  if (!workspace.includes(`- "${ROME_PROJECT}"`)) {
    await writeFile(
      workspacePath,
      `${workspace.trimEnd()}\n  - "${ROME_PROJECT}"\n`,
    );
  }

  const entryPath = path.join(targetProject, ROME_PROBLEMATIC_LIBS_ENTRY);
  await mkdir(path.dirname(entryPath), { recursive: true });
  await writeFile(
    entryPath,
    `import '../packages/@romejs/cli/cli';

import * as atlaskitEditorCore from '@atlaskit/editor-core';
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
    problematicLibrariesMarker,
    `${JSON.stringify(
      {
        entry: ROME_PROBLEMATIC_LIBS_ENTRY,
        dependencies: ROME_PROBLEMATIC_LIBRARIES,
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
await romeTsBenchcase();
await romeProblematicLibrariesBenchcase();
