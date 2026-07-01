#!/usr/bin/env node
import { $ } from 'zx';
import { mkdirSync, readdirSync, statSync, writeFileSync } from 'node:fs';
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

function* walk(dir) {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(file);
    else if (entry.isFile()) yield file;
  }
}

async function runCapture(command, args) {
  const result = await $({ nothrow: true })`${command} ${args}`;
  return {
    code: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

async function collectSymbols(target) {
  if (!target) return '';
  try {
    statSync(target);
  } catch {
    return '';
  }
  const result = await runCapture('nm', ['-C', '--defined-only', target]);
  if (result.code !== 0) return '';
  return result.stdout
    .split('\n')
    .map((line) => line.trim().split(/\s+/).slice(2).join(' '))
    .filter(Boolean)
    .join('\n');
}

function normalizeGeneric(name) {
  let prev;
  let current = name;
  do {
    prev = current;
    current = current.replace(/<[^<>]*>/g, '<...>');
  } while (current !== prev);
  return current;
}

async function runBloat(outDir) {
  const packageName = process.env.CARGO_BLOAT_PACKAGE || 'rspack_node';
  const crates = await runCapture('cargo', [
    'bloat',
    '--release',
    '-p',
    packageName,
    '--crates',
  ]);
  writeFileSync(path.join(outDir, 'cargo-bloat-crates.txt'), crates.stdout);
  writeFileSync(
    path.join(outDir, 'cargo-bloat-crates.stderr.txt'),
    crates.stderr,
  );
  const functions = await runCapture('cargo', [
    'bloat',
    '--release',
    '-p',
    packageName,
    '--functions',
  ]);
  writeFileSync(
    path.join(outDir, 'cargo-bloat-functions.txt'),
    functions.stdout,
  );
  writeFileSync(
    path.join(outDir, 'cargo-bloat-functions.stderr.txt'),
    functions.stderr,
  );
}

const root = await repoRoot();
const outDir = path.resolve(
  process.env.OUT_DIR ||
    path.join(
      root,
      'target',
      'binary-size-reports',
      `${stamp()}-generic-expansion`,
    ),
);
const [binary = ''] = positionalArgs();
mkdirSync(outDir, { recursive: true });
process.chdir(root);

let symbols = '';
if (binary) {
  symbols += await collectSymbols(binary);
} else {
  for (const file of walk(path.join(root, 'target'))) {
    if (
      file.endsWith('librspack_node.so') ||
      file.endsWith('.node') ||
      file.endsWith('.rlib')
    ) {
      symbols += `\n${await collectSymbols(file)}`;
    }
  }
}

writeFileSync(
  path.join(outDir, 'symbols.demangled.txt'),
  symbols.trim() ? `${symbols.trim()}\n` : '',
);

if (symbols.trim()) {
  const groups = new Map();
  for (const symbol of symbols
    .split('\n')
    .filter((line) => line.includes('<') && line.includes('>'))) {
    const normalized = normalizeGeneric(symbol);
    groups.set(normalized, (groups.get(normalized) ?? 0) + 1);
  }
  const groupRows = [...groups.entries()].sort(
    (a, b) => b[1] - a[1] || a[0].localeCompare(b[0]),
  );
  writeFileSync(
    path.join(outDir, 'generic-groups.tsv'),
    groupRows.map(([name, count]) => `${count}\t${name}`).join('\n') + '\n',
  );

  const patterns = [
    'ThreadsafeFunction<',
    'Function<',
    'Promise<',
    'Either<',
    'Either3<',
    'Either4<',
    'Either5<',
    'FromNapiValue',
    'ToNapiValue',
    'ValidateNapiValue',
    'TypeName',
    'CallbackInfo<',
    'Vec<',
    'Option<',
    'Result<',
  ];
  const markerLines = patterns.map(
    (pattern) => `${pattern}\t${symbols.split(pattern).length - 1}`,
  );
  writeFileSync(
    path.join(outDir, 'summary.txt'),
    `# Generic marker counts\n${markerLines.join('\n')}\n\n# Top normalized generic groups\n${groupRows
      .slice(0, 80)
      .map(([name, count]) => `${count}\t${name}`)
      .join('\n')}\n`,
  );
} else {
  writeFileSync(
    path.join(outDir, 'summary.txt'),
    `No symbols were collected.\n\nBuild an unstripped or profiling binary, or run this script after Rust artifacts exist in target/.\nStripped release binaries usually do not contain enough symbol information for generic expansion analysis.\n`,
  );
}

if (process.env.RUN_CARGO_BLOAT === '1') {
  await runBloat(outDir);
} else {
  writeFileSync(
    path.join(outDir, 'cargo-bloat.skipped.txt'),
    'cargo-bloat was skipped. Set RUN_CARGO_BLOAT=1 for section-level attribution.\n',
  );
}

console.log(`generic expansion report: ${outDir}`);
