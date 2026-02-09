#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';

const DEFAULT_CONFIG = path.resolve(
  'scripts/profile/bench-ts-react.config.cjs',
);

const rawArgs = process.argv.slice(2);
const args = rawArgs[0] === '--' ? rawArgs.slice(1) : rawArgs;

const { values, positionals } = parseArgs({
  args,
  options: {
    config: { type: 'string', default: DEFAULT_CONFIG },
    outDir: { type: 'string' },
    rate: { type: 'string', default: '99' },
    build: { type: 'boolean', default: false },
    trace: { type: 'boolean', default: true },
    traceFilter: { type: 'string', default: 'OVERVIEW' },
    perf: { type: 'string', default: 'perf' },
    node: { type: 'string', default: process.execPath },
    repeat: { type: 'string', default: '1' },
    addr2line: { type: 'string' },
    binding: { type: 'string' },
  },
  allowPositionals: true,
});

const timestamp = Date.now();
const outDir = path.resolve(
  values.outDir ?? `.rspack-profile-${timestamp}-${process.pid}`,
);
const configPath = path.resolve(values.config);
const rspackCli = path.resolve('packages/rspack-cli/bin/rspack.js');
const repeatCount = Number.parseInt(values.repeat, 10);
const bindingPath = path.resolve(
  values.binding ?? 'crates/node_binding/rspack.linux-x64-gnu.node',
);
let bindingMapInfo = null;

if (!Number.isFinite(repeatCount) || repeatCount < 1) {
  throw new Error(`Invalid repeat count: ${values.repeat}`);
}

function runCommand(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: 'inherit',
    ...options,
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`Command failed: ${command} ${args.join(' ')}`);
  }
}

function shellEscape(value) {
  return `'${value.replace(/'/g, "'\\''")}'`;
}

function runCapture(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    maxBuffer: 1024 * 1024 * 200,
    ...options,
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    const stderr = result.stderr ? `\n${result.stderr}` : '';
    throw new Error(`Command failed: ${command} ${args.join(' ')}${stderr}`);
  }
  return result.stdout ?? '';
}

function ensureCommand(command) {
  const result = spawnSync(command, ['--version'], { encoding: 'utf8' });
  if (result.error || result.status !== 0) {
    const errorMessage = result.error
      ? result.error.message
      : result.stderr || 'Unknown error';
    throw new Error(
      `Required tool "${command}" is not available. ${errorMessage}`,
    );
  }
}

function validateInputs() {
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`);
  }
  if (!fs.existsSync(rspackCli)) {
    throw new Error(
      `Rspack CLI not found at ${rspackCli}. Run "pnpm run build:js" first.`,
    );
  }
  if (!fs.existsSync(bindingPath)) {
    throw new Error(`Rspack binding not found at ${bindingPath}`);
  }
  ensureCommand(values.perf);
}

function runBuilds() {
  runCommand('pnpm', ['run', 'build:binding:profiling']);
  runCommand('pnpm', ['run', 'build:js']);
}

function writeMetadata(perfDataPath, reportPath, rawReportPath) {
  const metadata = {
    timestamp: new Date(timestamp).toISOString(),
    config: configPath,
    outputDir: outDir,
    perfDataPath,
    reportPath,
    rawReportPath,
    rspackCli,
    repeat: repeatCount,
    addr2line: values.addr2line ?? null,
    bindingPath,
    bindingMapInfo: bindingMapInfo
      ? {
          base: `0x${bindingMapInfo.base.toString(16)}`,
          offset: `0x${bindingMapInfo.offset.toString(16)}`,
        }
      : null,
    command: ['node', rspackCli, 'build', '-c', configPath, ...positionals],
  };
  fs.writeFileSync(
    path.join(outDir, 'profile-metadata.json'),
    JSON.stringify(metadata, null, 2),
  );
}

function runProfile() {
  fs.mkdirSync(outDir, { recursive: true });
  const perfDataPath = path.join(outDir, 'perf.data');
  const reportPath = path.join(outDir, 'line-report.txt');
  const tracePath = path.join(outDir, 'rspack.pftrace');

  const env = { ...process.env };
  if (values.trace) {
    env.RSPACK_PROFILE = values.traceFilter;
    env.RSPACK_TRACE_LAYER = 'perfetto';
    env.RSPACK_TRACE_OUTPUT = tracePath;
  }

  const nodeArgs = [
    '--perf-prof',
    '--perf-basic-prof',
    '--interpreted-frames-native-stack',
    rspackCli,
    'build',
    '-c',
    configPath,
    ...positionals,
  ];

  const perfArgs = [
    'record',
    '-F',
    values.rate,
    '--call-graph',
    'dwarf',
    '-o',
    perfDataPath,
    '--',
  ];

  if (repeatCount > 1) {
    const nodeCommand = [values.node, ...nodeArgs].map(shellEscape).join(' ');
    const loopCommand = `for i in $(seq 1 ${repeatCount}); do ${nodeCommand}; done`;
    perfArgs.push('bash', '-lc', loopCommand);
  } else {
    perfArgs.push(values.node, ...nodeArgs);
  }

  runCommand(values.perf, perfArgs, { env });

  const reportArgs = [
    'report',
    '--stdio',
    '--no-children',
    '--percent-limit',
    '0.5',
    '--sort',
    'dso,symbol',
    '--fields',
    'overhead,addr,symbol,dso',
    '--field-separator',
    '\t',
    '-i',
    perfDataPath,
  ];

  const rawReportPath = path.join(outDir, 'line-report.raw.txt');
  const reportOutput = runCapture(values.perf, reportArgs);
  fs.writeFileSync(rawReportPath, reportOutput);

  bindingMapInfo = getBindingMapInfo(perfDataPath);
  const resolvedReport = resolveReportLines(reportOutput);
  fs.writeFileSync(reportPath, resolvedReport);
  writeMetadata(perfDataPath, reportPath, rawReportPath);
  console.log(`Line-by-line report saved to ${reportPath}`);
  if (values.trace) {
    console.log(`Rspack trace saved to ${tracePath}`);
  }
}

function getBindingMapInfo(perfDataPath) {
  const output = runCapture(values.perf, [
    'script',
    '--show-mmap-events',
    '-i',
    perfDataPath,
  ]);
  const lines = output.split('\n');
  for (const line of lines) {
    if (!line.includes(bindingPath)) {
      continue;
    }
    const match = line.match(
      /\[(0x[0-9a-f]+)\(0x[0-9a-f]+\)\s+@\s+(0x[0-9a-f]+)[^\]]*\]:\s+(\S+)\s+(.+)$/i,
    );
    if (!match) {
      continue;
    }
    const [, base, offset, perms] = match;
    if (!perms.includes('r-x')) {
      continue;
    }
    return {
      base: BigInt(base),
      offset: BigInt(offset),
    };
  }
  return null;
}

function resolveReportLines(reportOutput) {
  const header = [
    'Overhead\tAddress\tSource:Line\tSymbol\tShared Object',
    '--------\t-------\t-----------\t------\t-------------',
  ];
  const lines = reportOutput
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => !line.startsWith('#'));
  const resolved = [];
  for (const line of lines) {
    const parts = line.split('\t');
    if (parts.length < 4) {
      continue;
    }
    const [overheadRaw, addressRaw, symbolRaw, dsoRaw] = parts;
    const overhead = overheadRaw.trim();
    const address = addressRaw.trim();
    const symbol = symbolRaw.trim();
    const dso = dsoRaw.trim();
    if (!overhead.endsWith('%')) {
      continue;
    }
    let sourceLine = '??:0';
    if (dso.endsWith(path.basename(bindingPath)) && values.addr2line) {
      sourceLine = resolveAddress(address, bindingMapInfo) ?? sourceLine;
    }
    resolved.push([overhead, address, sourceLine, symbol, dso].join('\t'));
  }
  return `${header.join('\n')}\n${resolved.join('\n')}\n`;
}

function resolveAddress(address, mapInfo) {
  if (!address || !values.addr2line || !mapInfo) {
    return null;
  }
  const clean = address.startsWith('0x') ? address : `0x${address}`;
  const relative = BigInt(clean) - mapInfo.base + mapInfo.offset;
  const relativeHex = `0x${relative.toString(16)}`;
  const result = spawnSync(values.addr2line, ['-e', bindingPath, relativeHex], {
    encoding: 'utf8',
  });
  if (result.error || result.status !== 0) {
    return null;
  }
  const output = result.stdout?.trim();
  return output && output !== '??:0' ? output.split('\n')[0] : null;
}

try {
  validateInputs();
  if (values.build) {
    runBuilds();
  }
  runProfile();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
