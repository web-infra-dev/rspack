#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';

const DEFAULT_CONFIG = path.resolve(
  'tests/bench/fixtures/ts-react/rspack.config.ts',
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
  },
  allowPositionals: true,
});

const timestamp = Date.now();
const outDir = path.resolve(
  values.outDir ?? `.rspack-profile-${timestamp}-${process.pid}`,
);
const configPath = path.resolve(values.config);
const rspackCli = path.resolve('packages/rspack-cli/bin/rspack.js');

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

  runCommand(
    values.perf,
    [
      'record',
      '-F',
      values.rate,
      '--call-graph',
      'dwarf',
      '-o',
      perfDataPath,
      '--',
      values.node,
      ...nodeArgs,
    ],
    { env },
  );

  const reportOutput = runCapture(values.perf, [
    'report',
    '--stdio',
    '--no-children',
    '--percent-limit',
    '0.5',
    '--sort',
    'dso,symbol,srcline',
    '--line',
    '-i',
    perfDataPath,
  ]);

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
