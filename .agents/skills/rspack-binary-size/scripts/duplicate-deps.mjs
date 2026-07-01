#!/usr/bin/env node
import TOML from '@iarna/toml';
import { $ } from 'zx';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
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
    String(value ?? '')
      .replaceAll('\t', ' ')
      .replaceAll('\n', ' ');
  writeFileSync(
    file,
    `${header.join('\t')}\n${rows.map((row) => row.map(escape).join('\t')).join('\n')}\n`,
  );
}

function parseCrateSpec(spec) {
  const text = String(spec ?? '').trim();
  const at = text.lastIndexOf('@');
  if (at > 0)
    return { name: text.slice(0, at), versionReq: text.slice(at + 1) };
  return { name: text, versionReq: '' };
}

function normalizeDenyEntry(kind, entry) {
  if (typeof entry === 'string') {
    const { name, versionReq } = parseCrateSpec(entry);
    return { kind, spec: entry, name, versionReq, reason: '' };
  }
  const spec = entry?.crate ?? '';
  const { name, versionReq } = parseCrateSpec(spec);
  return {
    kind,
    spec,
    name,
    versionReq,
    reason: entry?.reason ?? '',
  };
}

function readDenyConfig(root) {
  const denyPath = path.resolve(
    process.env.DENY_TOML || path.join(root, 'deny.toml'),
  );
  if (!existsSync(denyPath)) {
    return {
      denyPath,
      entries: [],
      error: `deny.toml not found at ${denyPath}`,
    };
  }
  try {
    const parsed = TOML.parse(readFileSync(denyPath, 'utf8'));
    const bans = parsed.bans ?? {};
    const entries = [
      ...(bans.skip ?? []).map((entry) => normalizeDenyEntry('skip', entry)),
      ...(bans['skip-tree'] ?? []).map((entry) =>
        normalizeDenyEntry('skip-tree', entry),
      ),
    ].filter((entry) => entry.name);
    return { denyPath, entries, error: '' };
  } catch (error) {
    return { denyPath, entries: [], error: error.message };
  }
}

function denyMatchesDuplicate(entry, duplicate) {
  if (!duplicate || entry.name !== duplicate.name) return false;
  if (!entry.versionReq) return true;
  return duplicate.versions.includes(entry.versionReq);
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

const metadataArgs = ['metadata', '--format-version', '1', '--locked'];
if (process.env.DENY_METADATA_ALL_FEATURES !== '0')
  metadataArgs.push('--all-features');
const metadataResult = await runCapture('cargo', metadataArgs);
writeFileSync(path.join(outDir, 'cargo-metadata.json'), metadataResult.stdout);
writeFileSync(
  path.join(outDir, 'cargo-metadata.stderr.txt'),
  metadataResult.stderr,
);

const deny = readDenyConfig(root);
const duplicateRows = [];
const duplicateMap = new Map();
const denySkipStatusRows = [];
const staleDenySkipRows = [];
const duplicateWithDenyRows = [];
let metadataSummary = `cargo ${metadataArgs.join(' ')} failed; duplicate-package-versions.tsv was not generated.`;

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

  for (const [name, set] of [...versions.entries()].sort((a, b) =>
    a[0].localeCompare(b[0]),
  )) {
    if (set.size <= 1) continue;
    const versionList = [...set].sort();
    duplicateRows.push([name, ...versionList]);
    duplicateMap.set(name, { name, versions: versionList });
  }

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

  for (const entry of deny.entries) {
    const duplicate = duplicateMap.get(entry.name);
    const status = denyMatchesDuplicate(entry, duplicate)
      ? 'still_duplicate'
      : 'remove_from_deny_toml';
    const row = [
      entry.kind,
      entry.spec,
      entry.name,
      entry.versionReq,
      entry.reason,
      status,
      duplicate?.versions.join(',') ?? '',
      deny.denyPath,
    ];
    denySkipStatusRows.push(row);
    if (status === 'remove_from_deny_toml') staleDenySkipRows.push(row);
  }

  for (const duplicate of duplicateMap.values()) {
    const matchingSkips = deny.entries.filter((entry) =>
      denyMatchesDuplicate(entry, duplicate),
    );
    duplicateWithDenyRows.push([
      duplicate.name,
      duplicate.versions.join(','),
      matchingSkips.length ? 'covered_by_deny_toml' : 'not_in_deny_toml',
      matchingSkips.map((entry) => `${entry.kind}:${entry.spec}`).join(','),
      matchingSkips
        .map((entry) => entry.reason)
        .filter(Boolean)
        .join(' | '),
    ]);
  }

  writeTsv(
    path.join(outDir, 'duplicate-package-versions-with-deny.tsv'),
    ['name', 'versions', 'deny_status', 'deny_entries', 'deny_reasons'],
    duplicateWithDenyRows.sort((a, b) => a[0].localeCompare(b[0])),
  );
  writeTsv(
    path.join(outDir, 'deny-skip-status.tsv'),
    [
      'kind',
      'spec',
      'name',
      'version_req',
      'reason',
      'status',
      'current_duplicate_versions',
      'deny_toml',
    ],
    denySkipStatusRows,
  );
  writeTsv(
    path.join(outDir, 'deny-skip-remove-candidates.tsv'),
    [
      'kind',
      'spec',
      'name',
      'version_req',
      'reason',
      'status',
      'current_duplicate_versions',
      'deny_toml',
    ],
    staleDenySkipRows,
  );

  metadataSummary = [
    `duplicate package names: ${duplicateRows.length}`,
    `deny.toml skip entries: ${deny.entries.length}`,
    `deny.toml skip entries still matching duplicates: ${denySkipStatusRows.length - staleDenySkipRows.length}`,
    `deny.toml skip entries to remove after fixes: ${staleDenySkipRows.length}`,
    `duplicate package names not covered by deny.toml: ${duplicateWithDenyRows.filter((row) => row[2] === 'not_in_deny_toml').length}`,
  ].join('\n');
}

if (deny.error) {
  writeFileSync(path.join(outDir, 'deny-toml.error.txt'), `${deny.error}\n`);
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
  `# Duplicate dependency report\npackage: ${packageName}\ndeny.toml: ${deny.denyPath}\nmetadata command: cargo ${metadataArgs.join(' ')}\n\n# Metadata and deny.toml status\n${metadataSummary}\n\n# cargo tree duplicate summary\n${duplicates.stdout.split('\n').slice(0, 160).join('\n')}\n\n# Common large dependency feature markers\n${markerLines.join('\n')}\n`,
);

console.log(`duplicate dependency report: ${outDir}`);
if (staleDenySkipRows.length > 0) {
  console.log(
    `deny.toml remove candidates: ${staleDenySkipRows.length} (see deny-skip-remove-candidates.tsv)`,
  );
}
