#!/usr/bin/env node
import { $ } from 'zx';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

$.quiet = true;

const skipDirs = new Set([
  '.git',
  'target',
  'node_modules',
  '.turbo',
  '.next',
  'dist',
  'coverage',
]);
const builtinAttrs = new Set([
  'allow',
  'cfg',
  'cfg_attr',
  'clippy',
  'cold',
  'deny',
  'deprecated',
  'doc',
  'expect',
  'forbid',
  'inline',
  'link',
  'must_use',
  'no_mangle',
  'non_exhaustive',
  'path',
  'repr',
  'rustfmt',
  'should_panic',
  'test',
  'warn',
]);

const attrRe =
  /#\s*!\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)|#\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)/g;
const callRe = /\b([A-Za-z_][A-Za-z0-9_:]*)!\s*[[({]/g;
const deriveRe = /#\s*\[\s*derive\s*\((.*?)\)\s*\]/gs;
const identRe = /[A-Za-z_][A-Za-z0-9_:]*/g;

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

function normalizeMacroName(name) {
  return name.split('::').at(-1);
}

function lineCol(text, offset) {
  let line = 0;
  let last = -1;
  for (let i = 0; i < offset; i++) {
    if (text.charCodeAt(i) === 10) {
      line++;
      last = i;
    }
  }
  return [line, offset - last - 1];
}

function lineAt(text, line) {
  return text.split(/\r?\n/)[line] ?? '';
}

function* walkRustFiles(root, scanRoots, maxBytes) {
  function* walk(dir) {
    let entries;
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      if (entry.isDirectory()) {
        if (!skipDirs.has(entry.name)) yield* walk(path.join(dir, entry.name));
      } else if (entry.isFile() && entry.name.endsWith('.rs')) {
        const file = path.join(dir, entry.name);
        try {
          if (statSync(file).size <= maxBytes) yield file;
        } catch {}
      }
    }
  }

  for (const scanRoot of scanRoots) {
    const base = path.join(root, scanRoot);
    if (existsSync(base)) yield* walk(base);
  }
}

function collectCandidates(root, scanRoots, maxBytes) {
  const candidates = [];
  const perMacro = new Map();

  function add(kind, name, file, line, character, snippet) {
    const normalized = normalizeMacroName(name);
    const rel = path.relative(root, file);
    const key = `${kind}\t${normalized}`;
    const current = perMacro.get(key) ?? {
      kind,
      name: normalized,
      count: 0,
      files: new Set(),
    };
    current.count++;
    current.files.add(rel);
    perMacro.set(key, current);
    candidates.push({
      kind,
      name: normalized,
      rawName: name,
      file,
      rel,
      line,
      character,
      snippet: snippet.trim().slice(0, 240),
    });
  }

  for (const file of walkRustFiles(root, scanRoots, maxBytes)) {
    const text = readFileSync(file, 'utf8');

    for (const match of text.matchAll(deriveRe)) {
      const body = match[1] ?? '';
      for (const item of body.matchAll(identRe)) {
        const [line, character] = lineCol(
          text,
          match.index + match[0].indexOf(body) + item.index,
        );
        add(
          'derive',
          item[0],
          file,
          line,
          character,
          match[0].replace(/\s+/g, ' '),
        );
      }
    }

    for (const match of text.matchAll(attrRe)) {
      const name = match[1] || match[2];
      if (!name) continue;
      const normalized = normalizeMacroName(name);
      if (builtinAttrs.has(normalized)) continue;
      const [line, character] = lineCol(text, match.index);
      add('attribute', name, file, line, character, lineAt(text, line));
    }

    for (const match of text.matchAll(callRe)) {
      const [line, character] = lineCol(text, match.index);
      add('function_like', match[1], file, line, character, lineAt(text, line));
    }
  }

  return { candidates, perMacro };
}

function writeTsv(file, header, rows) {
  const escape = (value) =>
    String(value).replaceAll('\t', ' ').replaceAll('\n', ' ');
  writeFileSync(
    file,
    `${header.join('\t')}\n${rows.map((row) => row.map(escape).join('\t')).join('\n')}\n`,
  );
}

function writeScanReports(outDir, candidates, perMacro) {
  const macroRows = [...perMacro.values()]
    .sort(
      (a, b) =>
        b.count - a.count ||
        a.kind.localeCompare(b.kind) ||
        a.name.localeCompare(b.name),
    )
    .map((item) => [item.kind, item.name, item.count, item.files.size]);
  writeTsv(
    path.join(outDir, 'macro-occurrences.tsv'),
    ['kind', 'name', 'count', 'files'],
    macroRows,
  );

  writeTsv(
    path.join(outDir, 'macro-locations.tsv'),
    ['kind', 'name', 'file', 'line', 'character', 'snippet'],
    candidates.map((item) => [
      item.kind,
      item.name,
      item.rel,
      item.line + 1,
      item.character + 1,
      item.snippet,
    ]),
  );

  const perFile = new Map();
  for (const item of candidates)
    perFile.set(item.rel, (perFile.get(item.rel) ?? 0) + 1);
  const fileRows = [...perFile.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([file, count]) => [count, file]);
  writeTsv(path.join(outDir, 'macro-files.tsv'), ['count', 'file'], fileRows);
}

class RustAnalyzerClient {
  constructor(root, analyzer) {
    this.root = root;
    this.nextId = 1;
    this.pending = new Map();
    this.notifications = [];
    this.errors = [];
    this.buffer = Buffer.alloc(0);
    this.proc = spawn(analyzer, [], {
      cwd: root,
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    this.proc.stdout.on('data', (chunk) => this.onData(chunk));
    this.proc.stderr.on('data', (chunk) => this.errors.push(chunk.toString()));
  }

  onData(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    while (true) {
      const headerEnd = this.buffer.indexOf('\r\n\r\n');
      if (headerEnd < 0) return;
      const header = this.buffer.slice(0, headerEnd).toString('ascii');
      const lengthMatch = /content-length:\s*(\d+)/i.exec(header);
      if (!lengthMatch) throw new Error(`missing Content-Length in ${header}`);
      const length = Number(lengthMatch[1]);
      const bodyStart = headerEnd + 4;
      if (this.buffer.length < bodyStart + length) return;
      const body = this.buffer
        .slice(bodyStart, bodyStart + length)
        .toString('utf8');
      this.buffer = this.buffer.slice(bodyStart + length);
      const message = JSON.parse(body);
      if (message.id && this.pending.has(message.id)) {
        const { resolve, reject } = this.pending.get(message.id);
        this.pending.delete(message.id);
        if (message.error) reject(new Error(JSON.stringify(message.error)));
        else resolve(message.result);
      } else {
        this.notifications.push(message);
      }
    }
  }

  send(message) {
    const body = Buffer.from(JSON.stringify(message), 'utf8');
    this.proc.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    this.proc.stdin.write(body);
  }

  request(method, params, timeoutMs = 60_000) {
    const id = this.nextId++;
    this.send({ jsonrpc: '2.0', id, method, params });
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`timed out waiting for ${method}`));
      }, timeoutMs);
      this.pending.set(id, {
        resolve: (value) => {
          clearTimeout(timeout);
          resolve(value);
        },
        reject: (error) => {
          clearTimeout(timeout);
          reject(error);
        },
      });
    });
  }

  notify(method, params) {
    this.send({ jsonrpc: '2.0', method, params });
  }

  async initialize() {
    const result = await this.request(
      'initialize',
      {
        processId: process.pid,
        rootUri: pathToFileURL(this.root).href,
        capabilities: {},
        workspaceFolders: [
          {
            uri: pathToFileURL(this.root).href,
            name: path.basename(this.root),
          },
        ],
      },
      120_000,
    );
    this.notify('initialized', {});
    return result;
  }

  async shutdown() {
    try {
      await this.request('shutdown', null, 10_000);
      this.notify('exit', null);
    } catch (error) {
      this.errors.push(`shutdown failed: ${error.message}`);
    } finally {
      this.proc.kill();
    }
  }
}

async function expandWithRustAnalyzer(root, outDir, candidates) {
  const analyzer = process.env.RUST_ANALYZER || 'rust-analyzer';
  const limit = Number(process.env.RA_EXPAND_LIMIT || 200);
  const filter = process.env.RA_EXPAND_FILTER
    ? new RegExp(process.env.RA_EXPAND_FILTER)
    : null;
  const timeoutMs = Number(process.env.RA_EXPAND_TIMEOUT || 45) * 1000;
  const selected = [];
  for (const item of candidates) {
    if (filter && !filter.test(item.name)) continue;
    selected.push(item);
    if (selected.length >= limit) break;
  }

  const expansionsDir = path.join(outDir, 'rust-analyzer-expansions');
  mkdirSync(expansionsDir, { recursive: true });
  const rows = [];
  const errors = [];
  const opened = new Set();
  const client = new RustAnalyzerClient(root, analyzer);
  try {
    await client.initialize();
    for (const item of selected) {
      const uri = pathToFileURL(item.file).href;
      if (!opened.has(uri)) {
        client.notify('textDocument/didOpen', {
          textDocument: {
            uri,
            languageId: 'rust',
            version: 1,
            text: readFileSync(item.file, 'utf8'),
          },
        });
        opened.add(uri);
      }
      try {
        const result = await client.request(
          'rust-analyzer/expandMacro',
          {
            textDocument: { uri },
            position: { line: item.line, character: item.character },
          },
          timeoutMs,
        );
        if (!result) {
          rows.push([
            'empty',
            item.kind,
            item.name,
            item.rel,
            item.line + 1,
            0,
            0,
            '',
          ]);
          continue;
        }
        const expansion =
          typeof result === 'object' ? result.expansion || '' : String(result);
        const macroName =
          typeof result === 'object' ? result.name || item.name : item.name;
        const digest = createHash('sha256')
          .update(expansion)
          .digest('hex')
          .slice(0, 16);
        const filename =
          `${String(rows.length).padStart(5, '0')}-${item.name}-${digest}.rs`.replaceAll(
            '/',
            '_',
          );
        writeFileSync(path.join(expansionsDir, filename), expansion);
        rows.push([
          'ok',
          item.kind,
          macroName,
          item.rel,
          item.line + 1,
          Buffer.byteLength(expansion),
          expansion ? expansion.split('\n').length : 0,
          filename,
        ]);
      } catch (error) {
        errors.push(
          `${item.rel}:${item.line + 1}:${item.character + 1} ${item.name}: ${error.message}`,
        );
        rows.push([
          'error',
          item.kind,
          item.name,
          item.rel,
          item.line + 1,
          0,
          0,
          '',
        ]);
      }
    }
  } finally {
    await client.shutdown();
  }
  writeTsv(
    path.join(outDir, 'rust-analyzer-expansions.tsv'),
    [
      'status',
      'kind',
      'name',
      'file',
      'line',
      'bytes',
      'lines',
      'expansion_file',
    ],
    rows,
  );
  if (errors.length || client.errors.length)
    writeFileSync(
      path.join(outDir, 'rust-analyzer-errors.txt'),
      `${[...errors, ...client.errors].join('\n')}\n`,
    );
}

async function runCapture(command, args) {
  const result = await $({ nothrow: true })`${command} ${args}`;
  return {
    code: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

async function cargoExpand(root, outDir) {
  const crates = (process.env.CARGO_EXPAND_CRATES || 'rspack_binding_api')
    .split(/\s+/)
    .filter(Boolean);
  const markers = [
    '__napi',
    'FromNapiValue',
    'ToNapiValue',
    'ValidateNapiValue',
    'TypeName',
    'CallbackInfo',
    'register_class',
    'register_module_export',
    'ThreadsafeFunction',
    'Either',
  ];
  const rows = [];
  for (const crate of crates) {
    const result = await runCapture('cargo', ['expand', '-p', crate]);
    writeFileSync(
      path.join(outDir, `cargo-expand-${crate}.stderr.txt`),
      result.stderr || '',
    );
    if (result.code !== 0) {
      rows.push([crate, 'error', 0, 0, '']);
      continue;
    }
    const expanded = result.stdout;
    writeFileSync(path.join(outDir, `cargo-expand-${crate}.rs`), expanded);
    const markerCounts = markers
      .map((marker) => `${marker}=${expanded.split(marker).length - 1}`)
      .join(',');
    rows.push([
      crate,
      'ok',
      Buffer.byteLength(expanded),
      expanded.split('\n').length,
      markerCounts,
    ]);
  }
  writeTsv(
    path.join(outDir, 'cargo-expand-summary.tsv'),
    ['crate', 'status', 'bytes', 'lines', 'marker_counts'],
    rows,
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
      `${stamp()}-macro-expansion`,
    ),
);
mkdirSync(outDir, { recursive: true });
process.chdir(root);

const scanRoots = (process.env.MACRO_SCAN_ROOTS || 'crates packages .agents')
  .split(/\s+/)
  .filter(Boolean);
const maxBytes = Number(process.env.MACRO_SCAN_MAX_BYTES || 1024 * 1024);
const { candidates, perMacro } = collectCandidates(root, scanRoots, maxBytes);
writeScanReports(outDir, candidates, perMacro);

const backend = process.env.MACRO_EXPAND_BACKEND || 'none';
if (backend === 'rust-analyzer') {
  try {
    await expandWithRustAnalyzer(root, outDir, candidates);
  } catch (error) {
    writeFileSync(
      path.join(outDir, 'rust-analyzer.failed.txt'),
      `${error.message}\n`,
    );
  }
} else if (backend === 'cargo-expand') {
  try {
    await cargoExpand(root, outDir);
  } catch (error) {
    writeFileSync(
      path.join(outDir, 'cargo-expand.failed.txt'),
      `${error.message}\n`,
    );
  }
} else if (backend !== 'none') {
  writeFileSync(
    path.join(outDir, 'backend.skipped.txt'),
    `unknown MACRO_EXPAND_BACKEND=${backend}\n`,
  );
}

console.log(`macro candidates: ${candidates.length}`);
console.log(`macro expansion report: ${outDir}`);
