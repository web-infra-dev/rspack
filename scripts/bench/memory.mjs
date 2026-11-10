import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import {
  mkdir,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { parseArgs } from 'node:util';
import vm from 'node:vm';

const root = path.resolve(import.meta.dirname, '../..');
const { values } = parseArgs({
  options: {
    baseline: { type: 'string' },
    candidate: { type: 'string' },
    binding: { type: 'string' },
    project: { type: 'string', default: 'threejs-10x' },
    mode: { type: 'string', default: 'production' },
    devtool: { type: 'string', default: 'source-map' },
    runs: { type: 'string', default: '5' },
    report: { type: 'string', default: '.bench/memory/results.json' },
    profile: { type: 'boolean', default: false },
    worker: { type: 'boolean', default: false },
  },
});
const project = path.resolve(root, '.bench/rspack-benchcases', values.project);
const output = path.join(root, '.bench/memory/output');
const marker = 'MEMORY_BENCH_RESULT ';
assert(['production', 'development'].includes(values.mode), 'Invalid mode');
assert(
  !process.env.NAPI_RS_FORCE_WASI,
  'This benchmark requires native bindings',
);
assert(
  values.profile || !process.env.RSPACK_ALLOCATIVE_DIR,
  'Use --profile for instrumented runs',
);

async function files(directory, prefix = '') {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = path.join(prefix, entry.name);
    if (entry.isDirectory())
      result.push(...(await files(path.join(directory, entry.name), relative)));
    else if (entry.isFile()) result.push(relative);
  }
  return result.sort();
}

async function digest(file) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}

if (values.worker) {
  assert(values.binding, '--binding is required for a worker');
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = path.resolve(values.binding);
  const require = createRequire(import.meta.url);
  const native = require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
  const coreRequire = createRequire(
    path.join(root, 'packages/rspack/dist/index.js'),
  );
  assert.equal(
    coreRequire('@rspack/binding'),
    native,
    'Core must use the selected binding',
  );
  const { rspack } = require('../../packages/rspack/dist/index.js');
  const phases = [];
  const sample = (phase) => {
    phases.push({
      phase,
      ...process.memoryUsage(),
      peakRssKiB: process.resourceUsage().maxRSS,
    });
  };
  const plugins = values.profile
    ? [
        {
          apply(compiler) {
            compiler.hooks.thisCompilation.tap(
              'MemoryBenchmark',
              (compilation) => {
                for (const hook of [
                  'finishModules',
                  'afterOptimizeModules',
                  'optimizeTree',
                  'optimizeChunkModules',
                  'afterSeal',
                ]) {
                  compilation.hooks[hook].tap('MemoryBenchmark', () =>
                    sample(hook),
                  );
                }
                for (const stage of [
                  -10000, 399, 401, 499, 501, 2499, 2501, 10000,
                ]) {
                  compilation.hooks.processAssets.tap(
                    { name: 'MemoryBenchmark', stage },
                    () => sample(`processAssets:${stage}`),
                  );
                }
              },
            );
            compiler.hooks.afterEmit.tap('MemoryBenchmark', () =>
              sample('afterEmit'),
            );
          },
        },
      ]
    : [];
  const compiler = rspack({
    context: project,
    mode: values.mode,
    entry: './src/index.js',
    devtool: values.devtool === 'false' ? false : values.devtool,
    output: { path: output, filename: 'main.js' },
    performance: false,
    plugins,
  });
  const started = performance.now();
  const stats = await new Promise((resolve, reject) =>
    compiler.run((error, stats) => {
      if (error) reject(error);
      else resolve(stats);
    }),
  );
  // Capture before stats materialization, output hashing and runtime verification.
  const wallMs = performance.now() - started;
  const usage = process.resourceUsage();
  const memory = process.memoryUsage();
  assert(
    stats && !stats.hasErrors(),
    stats?.toString({ all: false, errors: true }),
  );
  const moduleCount = stats.compilation.modules.size;
  const diagnostics = stats.toJson({
    all: false,
    errors: true,
    warnings: true,
  });
  await new Promise((resolve, reject) =>
    compiler.close((error) => (error ? reject(error) : resolve())),
  );
  const assets = [];
  for (const name of await files(output)) {
    const file = path.join(output, name);
    assets.push({
      name,
      bytes: (await stat(file)).size,
      sha256: await digest(file),
    });
  }
  assert(assets.length > 0, 'The build emitted no assets');
  let runtimeChecks = 0;
  if (values.project === 'threejs-10x') {
    const sandbox = {};
    vm.runInNewContext(
      await readFile(path.join(output, 'main.js'), 'utf8'),
      sandbox,
      { timeout: 30000 },
    );
    assert.equal(sandbox.__rspackThreejs10x.length, 10);
    for (const three of sandbox.__rspackThreejs10x) {
      assert.equal(new three.Vector3(1, 2, 3).lengthSq(), 14);
      assert.equal(new three.Vector3(1, 2, 3).multiplyScalar(2).lengthSq(), 56);
      runtimeChecks += 2;
    }
  }
  console.log(
    marker +
      JSON.stringify({
        peakRssKiB: usage.maxRSS,
        rssBytes: memory.rss,
        heapUsedBytes: memory.heapUsed,
        wallMs,
        moduleCount,
        errors: diagnostics.errors?.length ?? 0,
        warnings: diagnostics.warnings?.length ?? 0,
        runtimeChecks,
        assets,
        phases,
      }),
  );
} else {
  const runs = Number(values.runs);
  assert(
    Number.isInteger(runs) && runs > 0,
    '--runs must be a positive integer',
  );
  assert(
    values.baseline || values.binding,
    'Supply --baseline and --candidate, or --binding for a pilot',
  );
  assert(
    !values.baseline || values.candidate,
    '--baseline requires --candidate',
  );
  const bindings = values.baseline
    ? [
        ['baseline', values.baseline],
        ['candidate', values.candidate],
      ]
    : [['pilot', values.binding]];
  const environment = {
    ...process.env,
    RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS ?? '16',
    TOKIO_WORKER_THREADS: process.env.TOKIO_WORKER_THREADS ?? '16',
    RSPACK_BLOCKING_THREADS: process.env.RSPACK_BLOCKING_THREADS ?? '16',
  };
  const inputs = [];
  for (const name of await files(path.join(project, 'src'))) {
    const file = path.join(project, 'src', name);
    inputs.push([name, (await stat(file)).size, await digest(file)]);
  }
  const inputSha256 = createHash('sha256')
    .update(JSON.stringify(inputs))
    .digest('hex');
  const artifacts = {};
  for (const [label, binding] of bindings)
    artifacts[label] = {
      path: path.resolve(binding),
      sha256: await digest(binding),
    };
  if (values.baseline) {
    assert.notEqual(
      artifacts.baseline.sha256,
      artifacts.candidate.sha256,
      'Baseline and candidate must be different native artifacts',
    );
  }
  const results = [];
  let expectedAssets;
  let expectedModules;
  for (let iteration = 0; iteration < runs; iteration++) {
    const orderedBindings =
      iteration % 2 === 0 ? bindings : [...bindings].reverse();
    for (const [label, binding] of orderedBindings) {
      await rm(output, { recursive: true, force: true });
      const args = [
        import.meta.filename,
        '--worker',
        '--binding',
        path.resolve(binding),
        '--project',
        values.project,
        '--mode',
        values.mode,
        '--devtool',
        values.devtool,
        ...(values.profile ? ['--profile'] : []),
      ];
      const stdout = await new Promise((resolve, reject) => {
        const child = spawn(process.execPath, args, {
          cwd: root,
          env: environment,
          stdio: ['ignore', 'pipe', 'inherit'],
        });
        let text = '';
        child.stdout.setEncoding('utf8');
        child.stdout.on('data', (chunk) => {
          text += chunk;
        });
        child.on('error', reject);
        child.on('close', (code, signal) =>
          code === 0
            ? resolve(text)
            : reject(
                new Error(
                  `Build failed: code=${code} signal=${signal}\n${text}`,
                ),
              ),
        );
      });
      const line = stdout.split('\n').find((line) => line.startsWith(marker));
      assert(line, 'Worker did not report a completed build');
      const result = {
        label,
        iteration,
        ...JSON.parse(line.slice(marker.length)),
      };
      expectedAssets ??= result.assets;
      expectedModules ??= result.moduleCount;
      assert.deepEqual(result.assets, expectedAssets, 'Emitted assets differ');
      assert.equal(result.moduleCount, expectedModules, 'Module counts differ');
      assert(result.moduleCount > 0, 'No modules built');
      results.push(result);
      console.log(
        `${label} ${iteration + 1}/${runs}: ${(result.peakRssKiB / 1024).toFixed(1)} MiB peak RSS, ${result.wallMs.toFixed(0)} ms, ${result.moduleCount} modules, ${result.errors} errors`,
      );
    }
  }
  const summarize = (numbers) => {
    const sorted = [...numbers].sort((a, b) => a - b);
    const middle = Math.floor(sorted.length / 2);
    return {
      median:
        sorted.length % 2
          ? sorted[middle]
          : (sorted[middle - 1] + sorted[middle]) / 2,
      min: sorted[0],
      max: sorted.at(-1),
    };
  };
  const summary = {};
  for (const [label] of bindings) {
    const side = results.filter((result) => result.label === label);
    summary[label] = {
      peakRssKiB: summarize(side.map((result) => result.peakRssKiB)),
      wallMs: summarize(side.map((result) => result.wallMs)),
    };
  }
  const reductionPercent = values.baseline
    ? (1 -
        summary.candidate.peakRssKiB.median /
          summary.baseline.peakRssKiB.median) *
      100
    : null;
  const report = {
    capturedAt: new Date().toISOString(),
    node: process.version,
    platform: `${os.platform()} ${os.release()} ${os.arch()}`,
    cpu: os.cpus()[0].model,
    loadAverage: os.loadavg(),
    config: {
      project: values.project,
      mode: values.mode,
      devtool: values.devtool,
      runs,
      profile: values.profile,
      rayonThreads: environment.RAYON_NUM_THREADS,
      tokioThreads: environment.TOKIO_WORKER_THREADS,
      blockingThreads: environment.RSPACK_BLOCKING_THREADS,
    },
    inputs: {
      files: inputs.length,
      bytes: inputs.reduce((sum, file) => sum + file[1], 0),
      sha256: inputSha256,
    },
    measurement:
      'Fresh process high-water RSS at compiler.run completion, before stats/output validation; includes JS and native memory. Build wall time excludes compiler.close and validation. No forced GC or allocator purging.',
    artifacts,
    summary,
    reductionPercent,
    results,
  };
  const reportFile = path.resolve(values.report);
  await mkdir(path.dirname(reportFile), { recursive: true });
  await writeFile(reportFile, JSON.stringify(report, null, 2) + '\n');
  console.log(
    JSON.stringify({ summary, reductionPercent, report: reportFile }, null, 2),
  );
}
