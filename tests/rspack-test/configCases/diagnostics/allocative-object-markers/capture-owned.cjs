const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { rspack } = require('@rspack/core');
const binding = require('@rspack/binding');
const directory = process.env.RSPACK_ALLOCATIVE_DIR;
const sourceBytes = 1024 * 1024;
const requestBytes = 64 * 1024;

const run = compiler => new Promise((resolve, reject) => compiler.run(error => error ? reject(error) : resolve()));
const close = compiler => new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
const tick = () => new Promise(resolve => setImmediate(resolve));
function read(name) {
  return {
    native: fs.readFileSync(path.join(directory, `${name}.allocative`), 'utf8'),
    objects: JSON.parse(fs.readFileSync(path.join(directory, `${name}.objects.json`), 'utf8')).objects,
  };
}
function bytesFor(native, pattern) {
  return native.trim().split('\n').filter(line => pattern.test(line.slice(0, line.lastIndexOf(' '))))
    .reduce((sum, line) => sum + Number(line.slice(line.lastIndexOf(' ') + 1)), 0);
}
function measured(objects, type) {
  const values = objects.filter(object => object.type_name.endsWith(`::${type}`));
  assert(values.length > 0, `missing ${type}`);
  assert(values.every(object => object.measured), `unmeasured ${type}`);
}

(async () => {
  const options = { mode: 'development', context: __dirname, entry: './index.js', output: { path: directory } };
  const compiler = rspack(options);
  compiler.hooks.afterCompile.tap('RetainNativeOwners', compilation => {
    const source = binding.JsNativeSource.fromSource({ kind: 'raw', content: 'x'.repeat(sourceBytes) });
    const alias = binding.JsNativeSource.fromSource({ kind: 'native', handle: source });
    const diagnostic = binding.formatDiagnostic({ message: 'retained diagnostic', sourceCode: 'd'.repeat(32768), severity: 'warning' });
    compilation.__internal__pushDiagnostic(diagnostic);
    const automatic = new binding.AllocativeTestOwner(sourceBytes);
    const automaticFactory = binding.AllocativeTestOwner.fromOwner(automatic);
    const automaticAlias = automatic.alias();
    const payload = automatic.payload(128 * 1024);
    const payloadAlias = payload.alias();
    const payloadChained = payload.chained();
    const payloadWeak = payload.fromWeakOwner();
    const compilerId = compiler[Object.getOwnPropertySymbols(compiler).find(symbol => symbol.description === 'getCompilerId')]();
    assert.throws(() => compilation.__internal__pushDiagnostic(compilerId), /not the type of wrapped object/);
    const entries = compilation.__internal_getInner().entries;
    const entry = entries.get('main');
    globalThis.retained = {
      source, alias, diagnostic, automatic, automaticFactory, automaticAlias, payload, payloadAlias, payloadChained, payloadWeak,
      compilerId,
      errors: compilation.getWarnings(),
      dependency: new binding.EntryDependency('r'.repeat(requestBytes)),
      entries, entry, options: entry.options,
      graphDependency: entry.dependencies[0],
      graph: compilation.moduleGraph,
      stats: compilation.getStats(),
    };
  });
  try { await run(compiler); } finally { await close(compiler); }
  await tick();
  globalThis.gc();
  // Hold another native compiler inside a build hook while snapshots run.
  // Its registered wrapper must never be automatically dereferenced.
  const busy = rspack({ ...options, output: { path: path.join(directory, 'busy') } });
  let releaseBusy;
  let enteredBusy;
  const busyEntered = new Promise(resolve => { enteredBusy = resolve; });
  busy.hooks.make.tapAsync('HoldNativeBorrow', (_compilation, callback) => {
    releaseBusy = callback;
    enteredBusy();
  });
  const busyRun = run(busy);
  // Surface early build failures instead of waiting indefinitely for the hook.
  await Promise.race([busyEntered, busyRun.then(() => { throw new Error('busy hook did not run'); })]);
  const next = rspack(options);
  try {
    await run(next);
    const retained = read('1-build');
    const compilers = retained.objects.filter(object => object.type_name.endsWith('::JsCompiler'));
    assert(compilers.length >= 2, 'expected the active and busy compiler wrappers');
    assert.equal(compilers.filter(object => object.measured).length, 1, 'only the active compiler may be visited');
    const activeBytes = bytesFor(retained.native, /^rspack_binding_api::JsCompiler(?:;|$)/);
    assert(activeBytes > 0);
    for (const type of ['JsNativeSource', 'EntryDependency', 'EntryDataDTO', 'EntryOptionsDTO', 'Dependency', 'JsModuleGraph', 'JsStats']) measured(retained.objects, type);
    measured(retained.objects, 'AllocativeTestOwner');
    assert.equal(retained.objects.filter(object => object.type_name.endsWith('::AllocativeTestOwner')).length, 3);
    const automaticSize = bytesFor(retained.native, /::AllocativeTestOwner(?:;|$)/);
    assert(automaticSize >= sourceBytes && automaticSize < sourceBytes + 1024, `automatic shared owner counted incorrectly: ${automaticSize}`);
    const payloadSize = bytesFor(retained.native, /::TestPayload(?:;|$)/);
    assert(payloadSize >= 128 * 1024 && payloadSize < 128 * 1024 + 128, `generic shared payload missing or duplicated: ${payloadSize}`);
    assert.match(retained.native, /NapiOwnerAllocations;.*External.*CompilerId/, 'ordinary External should be registered without a type-specific call');
    const sourceSize = bytesFor(retained.native, /::JsNativeSource;/);
    assert(sourceSize >= sourceBytes && sourceSize < sourceBytes + 16384, `shared source counted incorrectly: ${sourceSize}`);
    const requestSize = bytesFor(retained.native, /::entry_dependency::EntryDependency;/);
    assert(requestSize >= requestBytes && requestSize < requestBytes + 1024);
    assert.match(retained.native, /NapiOwnerAllocations;.*External.*Diagnostic/);
    const diagnosticSize = bytesFor(retained.native, /^NapiOwnerAllocations;.*External.*Diagnostic/);
    assert(diagnosticSize >= 32768 && diagnosticSize < 32768 + 16384,
      `shared diagnostic counted incorrectly: ${diagnosticSize}`);
    for (const type of ['EntryDataDTO', 'JsEntries', 'ModuleHandle', 'JsStats']) {
      assert(!retained.native.split('\n').some(line =>
        line.includes(`::${type};`) && line.includes(';rspack_core::compilation::Compilation;')));
    }
    assert.doesNotMatch(fs.readFileSync(path.join(directory, '1-build.warnings.txt'), 'utf8'), /Incorrect size declaration/);
    assert(!retained.native.split('\n').some(line => line.includes('::JsStats;') && line.includes(';rspack_core::compilation::Compilation;')));
    // One explicit Stats plus the Stats produced for compiler.run share the same owner.
    // The payload is a borrowed Compilation pointer plus AtomicU8, rounded to alignment.
    const pointerBytes = process.arch === 'ia32' || process.arch === 'arm' ? 4 : 8;
    assert.equal(bytesFor(retained.native, /;payload;rspack_core::stats::Stats(?:<[^;]*>)?(?:;|$)/), 2 * 2 * pointerBytes);
    // JS Error owns a separate External; dropping formatDiagnostic's result
    // must not hide the remaining payload or keep the original External alive.
    globalThis.retained.diagnostic = undefined;
    globalThis.retained.payload = undefined;
    globalThis.retained.payloadAlias = undefined;
    globalThis.retained.payloadChained = undefined;
    globalThis.retained.payloadWeak = undefined;
    await tick();
    globalThis.gc();
    await run(next);
    const errorOnly = read('2-rebuild');
    assert.equal(bytesFor(errorOnly.native, /::TestPayload(?:;|$)/), payloadSize, 'owner must retain payload after both wrappers are collected');
    assert(!errorOnly.objects.some(object => object.type_name.endsWith('::AllocativeTestPayload')), 'profiling must not retain payload wrappers');
    const errorSize = bytesFor(errorOnly.native, /^NapiOwnerAllocations;.*External.*Diagnostic/);
    assert(errorSize >= 32768 && errorSize < diagnosticSize,
      `JS Error External missing or original External retained: ${errorSize}`);
    globalThis.retained = undefined;
    await tick();
    globalThis.gc();
    await run(next);
    const released = read('3-rebuild');
    for (const type of ['AllocativeTestOwner', 'JsNativeSource', 'EntryDependency', 'EntryDataDTO', 'EntryOptionsDTO']) {
      assert(!released.objects.some(object => object.type_name.endsWith(`::${type}`)), `${type} was retained by profiling`);
    }
    assert.doesNotMatch(released.native, /NapiOwnerAllocations;.*External.*Diagnostic/);
    assert.doesNotMatch(released.native, /::TestPayload(?:;|$)/m);
  } finally {
    releaseBusy();
    try { await busyRun; } finally { await close(busy); await close(next); }
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
