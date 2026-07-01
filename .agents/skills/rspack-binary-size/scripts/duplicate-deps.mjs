#!/usr/bin/env node
import { $ } from 'zx';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

$.quiet = true;

async function repoRoot() {
  try {
    return (await $`git rev-parse --show-toplevel`).stdout.trim();
  } catch {
    return process.cwd();
  }
}

function positionalArgs() {
  const args = process.argv.slice(2);
  const scriptPath = fileURLToPath(import.meta.url);
  if (args[0] && path.resolve(args[0]) === scriptPath) args.shift();
  if (args[0] === '--') args.shift();
  return args;
}

function stamp() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
}

async function runCapture(command, args) {
  const result = await $({ nothrow: true })`${command} ${args}`;
  return {
    code: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function writeTsv(file, header, rows) {
  const escape = (value) =>
    String(value).replaceAll('\t', ' ').replaceAll('\n', ' ');
  writeFileSync(
    file,
    `${header.join('\t')}\n${rows.map((row) => row.map(escape).join('\t')).join('\n')}\n`,
  );
}

const root = await repoRoot();
const [outDirArg] = positionalArgs();
const outDir = path.resolve(
  outDirArg ||
    path.join(
      root,
      'target',
      'binary-size-reports',
      `${stamp()}-duplicate-deps`,
    ),
);
const packageName = process.env.PACKAGE || 'rspack_node';
mkdirSync(outDir, { recursive: true });
process.chdir(root);

const duplicates = await runCapture('cargo', ['tree', '-p', packageName, '-d']);
writeFileSync(
  path.join(outDir, 'cargo-tree-duplicates.txt'),
  duplicates.stdout,
);
writeFileSync(
  path.join(outDir, 'cargo-tree-duplicates.stderr.txt'),
  duplicates.stderr,
);

const features = await runCapture('cargo', [
  'tree',
  '-p',
  packageName,
  '-e',
  'features',
]);
writeFileSync(path.join(outDir, 'cargo-tree-features.txt'), features.stdout);
writeFileSync(
  path.join(outDir, 'cargo-tree-features.stderr.txt'),
  features.stderr,
);

const metadataResult = await runCapture('cargo', [
  'metadata',
  '--format-version',
  '1',
  '--locked',
]);
writeFileSync(path.join(outDir, 'cargo-metadata.json'), metadataResult.stdout);
writeFileSync(
  path.join(outDir, 'cargo-metadata.stderr.txt'),
  metadataResult.stderr,
);

if (metadataResult.code === 0 && metadataResult.stdout.trim()) {
  const metadata = JSON.parse(metadataResult.stdout);
  const packageRows = metadata.packages
    .map((pkg) => [pkg.name, pkg.version, pkg.source || 'path'])
    .sort((a, b) => a.join('\t').localeCompare(b.join('\t')));
  writeTsv(
    path.join(outDir, 'packages.tsv'),
    ['name', 'version', 'source'],
    packageRows,
  );

  const versions = new Map();
  for (const [name, version] of packageRows.map((row) => [row[0], row[1]])) {
    const set = versions.get(name) ?? new Set();
    set.add(version);
    versions.set(name, set);
  }
  const duplicateRows = [...versions.entries()]
    .filter(([, set]) => set.size > 1)
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([name, set]) => [name, ...[...set].sort()]);
  writeTsv(
    path.join(outDir, 'duplicate-package-versions.tsv'),
    ['name', 'versions'],
    duplicateRows,
  );

  const featureRows = [];
  for (const node of metadata.resolve?.nodes ?? []) {
    for (const feature of node.features ?? [])
      featureRows.push([node.id, feature]);
  }
  writeTsv(
    path.join(outDir, 'resolved-features.tsv'),
    ['package_id', 'feature'],
    featureRows.sort((a, b) => a.join('\t').localeCompare(b.join('\t'))),
  );
}

const markers = [
  'napi',
  'napi-derive',
  'napi-sys',
  'tokio',
  'serde',
  'serde_json',
  'swc_core',
  'lightningcss',
  'rspack_plugin',
  'rspack_loader',
  'rspack_tracing',
];
const markerLines = markers.map(
  (marker) => `${marker}\t${features.stdout.split(marker).length - 1}`,
);
writeFileSync(
  path.join(outDir, 'summary.txt'),
  `# Duplicate dependency report\npackage: ${packageName}\n\n# cargo tree duplicate summary\n${duplicates.stdout.split('\n').slice(0, 160).join('\n')}\n\n# Common large dependency feature markers\n${markerLines.join('\n')}\n`,
);

console.log(`duplicate dependency report: ${outDir}`);
