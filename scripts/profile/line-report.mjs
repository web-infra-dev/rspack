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
  ensureCommand(values.perf);
}

function runBuilds() {
  runCommand('pnpm', ['run', 'build:binding:profiling']);
  runCommand('pnpm', ['run', 'build:js']);
}

function writeMetadata(perfDataPath, reportPath) {
  const metadata = {
    timestamp: new Date(timestamp).toISOString(),
    config: configPath,
    outputDir: outDir,
    perfDataPath,
    reportPath,
    rspackCli,
    repeat: repeatCount,
    addr2line: values.addr2line ?? null,
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
    'dso,symbol,srcline',
    '--fields',
    'overhead,srcline,symbol,dso',
  ];

  if (values.addr2line) {
    reportArgs.push('--addr2line', values.addr2line);
  }

  reportArgs.push('-i', perfDataPath);
  const reportOutput = runCapture(values.perf, reportArgs);

  fs.writeFileSync(reportPath, reportOutput);
  writeMetadata(perfDataPath, reportPath);
  console.log(`Line-by-line report saved to ${reportPath}`);
  if (values.trace) {
    console.log(`Rspack trace saved to ${tracePath}`);
  }
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
