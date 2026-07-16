#!/usr/bin/env zx

import { isUtf8 } from 'node:buffer';
import { readFile, readdir, stat } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createTwoFilesPatch } from 'diff';
import { argv, chalk } from 'zx';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const defaultRspackRoot = resolve(scriptDirectory, '../../../..');
const suiteDefinitions = [
  { webpack: 'benchmarkCases', rspack: 'benchmarkCases', depth: 1 },
  { webpack: 'cases', rspack: 'normalCases', depth: 2 },
  { webpack: 'configCases', rspack: 'configCases', depth: 2 },
  { webpack: 'hotCases', rspack: 'hotCases', depth: 2 },
  { webpack: 'memoryLimitCases', rspack: 'memoryLimitCases', depth: 1 },
  { webpack: 'statsCases', rspack: 'statsOutputCases', depth: 1 },
  { webpack: 'typesCases', rspack: 'typesCases', depth: 1 },
  { webpack: 'watchCases', rspack: 'watchCases', depth: 2 },
];
const ignoredDirectories = new Set([
  '__snapshots__',
  '_helpers',
  'node_modules',
]);

const help = [
  'Compare webpack and Rspack test case directories.',
  '',
  'Usage:',
  '  pnpm --filter @rspack/skill-webpack-gap-tracker diff-tests -- [filters...] [options]',
  '',
  'Arguments:',
  '  filters                Case-insensitive regular expressions matched against',
  '                         paths such as configCases/asset/source-map',
  '',
  'Options:',
  '  --webpack <path>       webpack repository or test directory',
  '                         (default: $WEBPACK_ROOT or ../webpack)',
  '  --rspack <path>        Rspack repository or rspack-test directory',
  '                         (default: the repository containing this skill)',
  '  --direction <value>    both, webpack, or rspack (default: both)',
  '  --content              Diff files in test cases present on both sides',
  '  --context <lines>      Unified diff context lines (default: 3)',
  '  -h, --help             Show this help',
].join('\n');

if (argv.help || argv.h) {
  console.log(help);
  process.exit(0);
}

const direction = String(argv.direction ?? 'both');
if (!['both', 'webpack', 'rspack'].includes(direction)) {
  throw new Error(
    'Invalid --direction ' +
      JSON.stringify(direction) +
      '. Expected both, webpack, or rspack.',
  );
}

const showContent = Boolean(argv.content);
const contextLines = Number(argv.context ?? 3);
if (!Number.isInteger(contextLines) || contextLines < 0) {
  throw new Error(
    'Invalid --context ' +
      JSON.stringify(argv.context) +
      '. Expected a non-negative integer.',
  );
}

const rspackInput = resolve(String(argv.rspack ?? defaultRspackRoot));
const webpackInput = resolve(
  String(
    argv.webpack ??
      process.env.WEBPACK_ROOT ??
      resolve(rspackInput, '../webpack'),
  ),
);

const rspackTests = await resolveTestDirectory(
  rspackInput,
  'tests/rspack-test',
  'Rspack',
  'rspack',
);
const webpackTests = await resolveTestDirectory(
  webpackInput,
  'test',
  'webpack',
  'webpack',
);
const filters = compileFilters(argv._.map(String));

const [webpackCases, rspackCases] = await Promise.all([
  collectCases(webpackTests, 'webpack'),
  collectCases(rspackTests, 'rspack'),
]);

const webpackOnly = difference(webpackCases, rspackCases, filters);
const rspackOnly = difference(rspackCases, webpackCases, filters);

console.log(chalk.bold('Test directory diff'));
console.log('webpack: ' + webpackTests);
console.log('rspack:  ' + rspackTests);
if (filters.length > 0) {
  console.log('filters: ' + argv._.join(', '));
}

if (direction === 'both' || direction === 'webpack') {
  printGrouped('webpack only (missing from Rspack)', webpackOnly);
}
if (direction === 'both' || direction === 'rspack') {
  printGrouped('Rspack only', rspackOnly);
}

if (showContent) {
  await printContentDiff({
    webpackCases,
    rspackCases,
    filters,
    direction,
    contextLines,
  });
}

async function resolveTestDirectory(input, nestedPath, label, source) {
  const nested = join(input, nestedPath);
  if (await isDirectory(nested)) return nested;
  if ((await isDirectory(input)) && (await containsKnownSuite(input, source))) {
    return input;
  }

  throw new Error(
    label + ' tests were not found. Checked ' + input + ' and ' + nested + '.',
  );
}

async function containsKnownSuite(path, source) {
  const entries = await readdir(path, { withFileTypes: true });
  const suiteNames = new Set(
    suiteDefinitions.map((definition) => definition[source]),
  );
  return entries.some(
    (entry) => entry.isDirectory() && suiteNames.has(entry.name),
  );
}

async function isDirectory(path) {
  try {
    return (await stat(path)).isDirectory();
  } catch (error) {
    if (error?.code === 'ENOENT') return false;
    throw error;
  }
}

async function collectCases(testRoot, source) {
  const cases = new Map();

  for (const definition of suiteDefinitions) {
    const suite = definition[source];
    const suiteRoot = join(testRoot, suite);
    if (!(await isDirectory(suiteRoot))) continue;

    for (const casePath of await directoriesAtDepth(
      suiteRoot,
      definition.depth,
    )) {
      cases.set(definition.webpack + '/' + casePath, join(suiteRoot, casePath));
    }
  }

  return cases;
}

async function directoriesAtDepth(root, depth) {
  const results = [];

  async function visit(current, remainingDepth) {
    const entries = (await readdir(current, { withFileTypes: true }))
      .filter(
        (entry) =>
          entry.isDirectory() &&
          !entry.name.startsWith('.') &&
          !ignoredDirectories.has(entry.name),
      )
      .sort((a, b) => a.name.localeCompare(b.name));

    for (const entry of entries) {
      const path = join(current, entry.name);
      if (remainingDepth === 1) {
        results.push(relative(root, path).split(sep).join('/'));
      } else {
        await visit(path, remainingDepth - 1);
      }
    }
  }

  await visit(root, depth);
  return results;
}

function compileFilters(rawFilters) {
  return rawFilters.map((filter) => {
    try {
      return new RegExp(filter.replaceAll('\\', '/'), 'i');
    } catch (error) {
      throw new Error(
        'Invalid filter ' + JSON.stringify(filter) + ': ' + error.message,
      );
    }
  });
}

function matchesFilters(path, activeFilters) {
  return (
    activeFilters.length === 0 ||
    activeFilters.some((filter) => filter.test(path))
  );
}

function difference(left, right, activeFilters) {
  return [...left.keys()]
    .filter((path) => !right.has(path))
    .filter((path) => matchesFilters(path, activeFilters))
    .sort((a, b) => a.localeCompare(b));
}

async function printContentDiff({
  webpackCases,
  rspackCases,
  filters,
  direction,
  contextLines,
}) {
  const sharedCases = [...webpackCases.keys()]
    .filter((path) => rspackCases.has(path))
    .filter((path) => matchesFilters(path, filters))
    .sort((a, b) => a.localeCompare(b));
  const groupedDiffs = [];
  let fileCount = 0;

  for (const casePath of sharedCases) {
    const diffs = await diffCaseFiles({
      casePath,
      webpackRoot: webpackCases.get(casePath),
      rspackRoot: rspackCases.get(casePath),
      direction,
      contextLines,
    });
    if (diffs.length > 0) {
      groupedDiffs.push({ casePath, diffs });
      fileCount += diffs.length;
    }
  }

  console.log(
    '\n' +
      chalk.bold(
        'File content diff (' +
          groupedDiffs.length +
          ' cases, ' +
          fileCount +
          ' files)',
      ),
  );
  if (fileCount === 0) {
    console.log('  (none)');
    return;
  }

  for (const { casePath, diffs } of groupedDiffs) {
    console.log('\n' + chalk.bold(casePath + '/'));
    for (const diff of diffs) {
      console.log(chalk.dim(diff.label));
      console.log(diff.patch);
    }
  }
}

async function diffCaseFiles({
  casePath,
  webpackRoot,
  rspackRoot,
  direction,
  contextLines,
}) {
  const [webpackFiles, rspackFiles] = await Promise.all([
    collectFiles(webpackRoot, 'webpack'),
    collectFiles(rspackRoot, 'rspack'),
  ]);
  const fileKeys = new Set([...webpackFiles.keys(), ...rspackFiles.keys()]);
  const diffs = [];

  for (const key of [...fileKeys].sort((a, b) => a.localeCompare(b))) {
    const webpackFile = webpackFiles.get(key);
    const rspackFile = rspackFiles.get(key);
    if (!rspackFile && direction === 'rspack') continue;
    if (!webpackFile && direction === 'webpack') continue;

    const diff = await createFileDiff({
      casePath,
      webpackFile,
      rspackFile,
      contextLines,
    });
    if (diff) diffs.push(diff);
  }

  return diffs;
}

async function collectFiles(root, source) {
  const files = new Map();

  async function visit(current) {
    const entries = (await readdir(current, { withFileTypes: true })).sort(
      (a, b) => a.name.localeCompare(b.name),
    );
    for (const entry of entries) {
      if (entry.name === '.git') continue;
      const path = join(current, entry.name);
      if (entry.isDirectory()) {
        await visit(path);
      } else if (entry.isFile()) {
        const relativePath = relative(root, path).split(sep).join('/');
        const key = canonicalFilePath(relativePath, source);
        if (files.has(key)) {
          throw new Error(
            'Multiple ' + source + ' files map to ' + key + ' in ' + root + '.',
          );
        }
        files.set(key, { path, relativePath });
      }
    }
  }

  await visit(root);
  return files;
}

function canonicalFilePath(path, source) {
  const parts = path.split('/');
  const name = parts.at(-1);
  const configPrefix = source === 'webpack' ? 'webpack' : 'rspack';
  if (new RegExp('^' + configPrefix + '\\.config\\.[cm]?[jt]s$').test(name)) {
    parts[parts.length - 1] = '__bundler_config__';
  }
  return parts.join('/');
}

async function createFileDiff({
  casePath,
  webpackFile,
  rspackFile,
  contextLines,
}) {
  const [webpackBuffer, rspackBuffer] = await Promise.all([
    webpackFile ? readFile(webpackFile.path) : undefined,
    rspackFile ? readFile(rspackFile.path) : undefined,
  ]);
  if (webpackBuffer && rspackBuffer && webpackBuffer.equals(rspackBuffer))
    return undefined;

  const webpackName = displayFileName('webpack', casePath, webpackFile);
  const rspackName = displayFileName('rspack', casePath, rspackFile);
  const label =
    webpackFile?.relativePath === rspackFile?.relativePath
      ? (webpackFile ?? rspackFile).relativePath
      : (webpackFile?.relativePath ?? '/dev/null') +
        ' -> ' +
        (rspackFile?.relativePath ?? '/dev/null');

  if (
    (webpackBuffer && !isUtf8(webpackBuffer)) ||
    (rspackBuffer && !isUtf8(rspackBuffer))
  ) {
    return {
      label,
      patch: 'Binary files differ: ' + webpackName + ' and ' + rspackName,
    };
  }

  return {
    label,
    patch: createTwoFilesPatch(
      webpackName,
      rspackName,
      webpackBuffer?.toString('utf8') ?? '',
      rspackBuffer?.toString('utf8') ?? '',
      '',
      '',
      { context: contextLines },
    ).trimEnd(),
  };
}

function displayFileName(source, casePath, file) {
  if (!file) return '/dev/null';
  return (
    source + '/' + sourceCasePath(casePath, source) + '/' + file.relativePath
  );
}

function sourceCasePath(casePath, source) {
  const definition = suiteDefinitions.find(
    (item) =>
      casePath === item.webpack || casePath.startsWith(item.webpack + '/'),
  );
  if (!definition) return casePath;
  return definition[source] + casePath.slice(definition.webpack.length);
}

function printGrouped(label, paths) {
  console.log('\n' + chalk.bold(label + ' (' + paths.length + ')'));
  if (paths.length === 0) {
    console.log('  (none)');
    return;
  }

  let previousDirectory;
  for (const path of paths) {
    const lastSlash = path.lastIndexOf('/');
    const directory = path.slice(0, lastSlash);
    const caseName = path.slice(lastSlash + 1);
    if (directory !== previousDirectory) {
      console.log(directory + '/');
      previousDirectory = directory;
    }
    console.log('  - ' + caseName);
  }
}
