#!/usr/bin/env zx

import * as fs from 'node:fs/promises';
import path from 'node:path';
import { $, argv, which } from 'zx';

async function readBytesSnapshot(filename) {
  if (path.extname(filename) !== '.allocative') {
    throw new Error('Expected a byte-weighted .allocative file');
  }
  const text = await fs.readFile(filename, 'utf8');
  let total = 0;
  let rows = 0;
  for (const [index, line] of text.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    const match = /^(\S.*) ([0-9]+)$/.exec(line);
    const weight = match ? Number(match[2]) : NaN;
    if (
      !Number.isSafeInteger(weight) ||
      !Number.isSafeInteger(total + weight)
    ) {
      throw new Error(
        `Invalid folded row ${index + 1}: expected safe integer bytes`,
      );
    }
    total += weight;
    rows++;
  }
  if (!total) {
    throw new Error(
      'Empty or zero-byte snapshot: check root coverage and capture settings',
    );
  }
  return { text, total, rows };
}

async function readCounts(filename, snapshot) {
  const data = JSON.parse(await fs.readFile(filename, 'utf8'));
  if (!data || Array.isArray(data) || data.snapshot !== snapshot) {
    throw new Error('Counts snapshot must match the .allocative filename stem');
  }
  if (!['objects', 'entries'].includes(data.unit)) {
    throw new Error('Counts unit must be objects or entries; do not mix units');
  }
  if (typeof data.scope !== 'string' || !data.scope.trim()) {
    throw new Error('Counts must specify a nonempty scope');
  }
  if (!Array.isArray(data.counts) || !data.counts.length) {
    throw new Error('Counts must contain at least one measured population');
  }
  const paths = new Set();
  let total = 0;
  for (const row of data.counts) {
    if (!row || !Array.isArray(row.path) || !row.path.length) {
      throw new Error('Each population needs a nonempty path array');
    }
    if (
      row.path.some(
        (label) =>
          typeof label !== 'string' ||
          !label.trim() ||
          label !== label.trim() ||
          /[;\r\n\t]/.test(label),
      )
    ) {
      throw new Error(
        'Path labels must be nonempty, trimmed, and contain no ; or line breaks',
      );
    }
    const key = row.path.join(';');
    if (paths.has(key)) throw new Error(`Duplicate population: ${key}`);
    paths.add(key);
    if (
      !Number.isSafeInteger(row.count) ||
      row.count < 0 ||
      !Number.isSafeInteger(total + row.count)
    ) {
      throw new Error(
        'Counts and their total must be nonnegative safe integers',
      );
    }
    total += row.count;
    if (typeof row.basis !== 'string' || !row.basis.trim()) {
      throw new Error('Each population needs its measured counting basis');
    }
  }
  for (const row of data.counts) {
    for (let i = 1; i < row.path.length; i++) {
      if (paths.has(row.path.slice(0, i).join(';'))) {
        throw new Error(
          'Ancestor and descendant populations would double-count totals',
        );
      }
    }
  }
  return { ...data, total };
}

async function render(inferno, folded, filename, unit, title) {
  const result = await $({
    quiet: true,
    input: folded,
    timeout: '120s',
  })`${inferno} --countname ${unit} --nametype Object: --title ${title} --deterministic`;
  if (!result.stdout.includes('<svg'))
    throw new Error('Inferno did not produce an SVG');
  await fs.writeFile(filename, result.stdout);
}

function csvRow(values) {
  return (
    values
      .map((value) => `"${String(value).replaceAll('"', '""')}"`)
      .join(',') + '\n'
  );
}

async function main() {
  const values = { inferno: 'inferno-flamegraph', ...argv };
  const positionals = argv._;
  const known = new Set(['_', 'counts', 'out-dir', 'inferno', 'help', 'h']);
  for (const key of Object.keys(argv)) {
    if (!known.has(key)) throw new Error(`Unknown option: ${key}`);
  }
  if (values.help || values.h) {
    console.log(
      'Usage: zx render-profile.mjs SNAPSHOT.allocative --out-dir NEW_DIR [--counts COUNTS.json] [--inferno BINARY]',
    );
    return;
  }
  if (positionals.length !== 1 || !values['out-dir']) {
    throw new Error(
      'Supply one .allocative snapshot and --out-dir NEW_DIR; see --help',
    );
  }
  for (const key of ['counts', 'out-dir', 'inferno']) {
    if (values[key] !== undefined && typeof values[key] !== 'string') {
      throw new Error(`--${key} requires a single string value`);
    }
  }
  const source = path.resolve(positionals[0]);
  const snapshot = path.basename(source, '.allocative');
  const bytes = await readBytesSnapshot(source);
  const counts = values.counts
    ? await readCounts(values.counts, snapshot)
    : null;
  const inferno = await which(values.inferno).catch(() => {
    throw new Error(
      'inferno-flamegraph not found; install Inferno or pass --inferno',
    );
  });
  const out = path.resolve(values['out-dir']);
  await fs.mkdir(path.dirname(out), { recursive: true });
  await fs.mkdir(out); // Refuse to overwrite a previous report.
  await fs.writeFile(path.join(out, path.basename(source)), bytes.text);
  await render(
    inferno,
    bytes.text,
    path.join(out, 'memory.svg'),
    'bytes',
    `Allocative bytes: ${snapshot}`,
  );
  const summary = {
    snapshot,
    source,
    bytes: {
      visited_bytes: bytes.total,
      folded_rows: bytes.rows,
      svg: path.join(out, 'memory.svg'),
      coverage: 'Only visited allocative roots/fields',
    },
    counts: null,
  };
  if (counts) {
    await fs.copyFile(values.counts, path.join(out, 'counts.json'));
    const rows = [...counts.counts].sort(
      (a, b) =>
        b.count - a.count || a.path.join(';').localeCompare(b.path.join(';')),
    );
    const folded = rows
      .filter((row) => row.count)
      .map((row) => `${row.path.join(';')} ${row.count}\n`)
      .join('');
    await fs.writeFile(path.join(out, 'counts.folded'), folded);
    const csv =
      csvRow(['population', 'count', 'unit', 'basis']) +
      rows
        .map((row) =>
          csvRow([row.path.join(' / '), row.count, counts.unit, row.basis]),
        )
        .join('');
    await fs.writeFile(path.join(out, 'counts.csv'), csv);
    if (counts.total) {
      await render(
        inferno,
        folded,
        path.join(out, 'counts.svg'),
        counts.unit,
        `Scoped ${counts.unit}: ${snapshot}`,
      );
    }
    summary.counts = {
      unit: counts.unit,
      scope: counts.scope,
      total: counts.total,
      populations: rows.length,
      table: path.join(out, 'counts.csv'),
      svg: counts.total ? path.join(out, 'counts.svg') : null,
      note: 'Sum of declared nonoverlapping populations; not a process-wide census',
    };
  }
  await fs.writeFile(
    path.join(out, 'summary.json'),
    JSON.stringify(summary, null, 2) + '\n',
  );
  console.log(JSON.stringify(summary, null, 2));
}

try {
  await main();
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exitCode = 1;
  throw error;
}
