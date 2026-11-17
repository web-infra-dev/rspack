import assert from 'node:assert/strict';
import { verifyV8Filter } from './verify-v8-filter.mjs';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { convert } from '../../../../../.agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs';
import { parseHeapSnapshot, readHeapGraph, readBindingMarker } from '../../../../../.agents/skills/rspack-allocative/scripts/heapSnapshot.mjs';

function verifyStringPreviews() {
  const nativeType = 'foreign_binding_crate::' + 'LongNativeType'.repeat(12);
  const pointer = '0x123456789abcdef0';
  const original = {
    snapshot: {
      meta: {
        node_fields: ['type', 'name', 'id', 'self_size', 'edge_count'],
        node_types: [['synthetic', 'object', 'string'], 'string', 'number', 'number', 'number'],
        edge_fields: ['type', 'name_or_index', 'to_node'],
        edge_types: [['property', 'element', 'internal', 'hidden'], 'string_or_number', 'node'],
      },
      node_count: 7,
      edge_count: 6,
    },
    nodes: [0, 0, 1, 0, 1, 1, 1, 3, 64, 5, 2, 7, 5, 10000, 0, 2, 3, 7, 200, 0, 2, 6, 9, 40, 0, 2, 8, 11, 50, 0, 2, 9, 13, 60, 0],
    edges: [1, 8, 5, 0, 2, 10, 0, 5, 15, 0, 4, 20, 0, 10, 25, 0, 11, 30],
    // Identity values intentionally precede their property-name entries.
    strings: ['', 'Constructor' + 'Name'.repeat(20), 'property' + 'Key'.repeat(20), nativeType,
      '__rspackRustPtr', '__rspackRustType', pointer,
      'a'.repeat(31) + '😀' + 'x'.repeat(4096), 'b'.repeat(32) + 'left', 'b'.repeat(32) + 'right', 'left', 'right'],
  };
  function parse(options) {
    const bytes = Buffer.from(JSON.stringify(original));
    let offset = 0;
    return parseHeapSnapshot(() => {
      if (offset === bytes.length) return null;
      const chunk = bytes.subarray(offset, offset + 7);
      offset += chunk.length;
      return chunk;
    }, options);
  }
  const full = parse();
  assert.deepEqual(full.strings, original.strings);
  const preview = parse({ maxStringLength: 32, preserveStrings: [nativeType, pointer] });
  const graph = readHeapGraph(preview);
  assert.deepEqual(preview.rspack_string_preview, { max_length: 32, truncated_strings: 3 });
  assert.equal(preview.strings[7], 'a'.repeat(31) + '…');
  assert.equal(preview.strings[8], 'b'.repeat(32) + '…');
  assert.equal(preview.strings[9], preview.strings[8]);
  assert.equal(graph.nodeName(1), original.strings[1]);
  assert.equal(graph.edgeName(3), original.strings[2]);
  assert.deepEqual(readBindingMarker(graph, 1), [3, pointer, nativeType]);
  assert.deepEqual(Array.from(preview.nodes), original.nodes);
  assert.deepEqual(Array.from(preview.edges), original.edges);
  const identities = { objects: [{ address: pointer, type_name: nativeType, path: ['wrapper'], measured: true }] };
  const before = convert(full, identities);
  const after = convert(preview, identities);
  assert.equal(after.report.matched_bindings, 1);
  assert.deepEqual(after.report.category_bytes, before.report.category_bytes);
  assert.equal(after.report.represented_bytes, before.report.represented_bytes);
  assert.equal(after.report.node_count, before.report.node_count);
  assert.equal(after.folded, before.folded);
  const named = convert(preview, identities);
  assert(named.folded.includes(original.strings[1]));
  assert(!named.folded.includes('b'.repeat(32)));
  assert(!named.folded.includes(nativeType));
  assert(!named.folded.includes(pointer));
  assert(!named.folded.includes('x'.repeat(100)));
  const omitted = parse({ omitStringContents: true, preserveStrings: [nativeType, pointer] });
  assert.deepEqual(omitted.strings.slice(7, 10), ['', '', '']);
  assert.deepEqual(omitted.rspack_string_contents, { omitted_strings: 3 });
  assert.deepEqual(readBindingMarker(readHeapGraph(omitted), 1), [3, pointer, nativeType]);
  const sized = convert(omitted, identities);
  assert.equal(sized.folded, before.folded);
  assert.deepEqual(sized.report.category_bytes, before.report.category_bytes);
  assert.equal(sized.report.matched_bindings, 1);
  const tiny = parse({ maxStringLength: 1, preserveStrings: [nativeType, pointer] });
  assert.deepEqual(readBindingMarker(readHeapGraph(tiny), 1), [3, pointer, nativeType]);
  assert.throws(() => parse({ maxStringLength: 0 }), /positive integer/);
}

function verifyObjectNames() {
  const builtins = ['Object', 'Array', 'Map', 'Set', 'Promise', 'Uint8Array', 'Buffer'];
  const strings = ['', 'SensitivePlugin', 'PRIVATE_STRING_BODY_DO_NOT_EMIT', ...builtins];
  const objects = [{ type: 1, name: 1 }, ...builtins.map((_, i) => ({ type: 1, name: i + 3 })),
    { type: 3, name: 1 }, { type: 4, name: 1 }, { type: 2, name: 2 }];
  const heap = {
    snapshot: { meta: {
      node_fields: ['type', 'name', 'id', 'self_size', 'edge_count'],
      node_types: [['synthetic', 'object', 'string', 'closure', 'code'], 'string', 'number', 'number', 'number'],
      edge_fields: ['type', 'name_or_index', 'to_node'],
      edge_types: [['element'], 'string_or_number', 'node'],
    } },
    nodes: [0, 0, 1, 0, objects.length],
    edges: [], strings,
  };
  for (const [i, object] of objects.entries()) {
    heap.nodes.push(object.type, object.name, i * 2 + 3, 64, 0);
    heap.edges.push(0, i, (i + 1) * 5);
  }
  const named = convert(heap, { objects: [] });
  const cleared = convert(heap, { objects: [] }, { clearClassNames: true });
  assert(named.folded.includes('object: SensitivePlugin'));
  assert(named.folded.includes('closure: SensitivePlugin'));
  assert(!named.folded.includes('code: SensitivePlugin'));
  assert.equal(named.report.snapshot_bytes, named.report.represented_bytes + named.report.excluded_v8_bytes);
  assert(!cleared.folded.includes('SensitivePlugin'));
  for (const name of builtins) {
    assert(named.folded.includes('object: ' + name));
    assert(cleared.folded.includes('object: ' + name));
  }
  assert(!named.folded.includes('PRIVATE_STRING_BODY_DO_NOT_EMIT'));
  assert(!cleared.folded.includes('PRIVATE_STRING_BODY_DO_NOT_EMIT'));
  assert.deepEqual(named.report.category_bytes, cleared.report.category_bytes);
  assert.equal(named.report.represented_bytes, cleared.report.represented_bytes);
  assert.equal(named.report.node_count, cleared.report.node_count);
  assert.equal(named.report.string_contents, 'omitted');
  assert.equal(cleared.report.clear_class_names, true);
}

export default {
  afterExecute() {
    verifyV8Filter();
    verifyStringPreviews();
    verifyObjectNames();
    if (process.env.RSPACK_BUILD_PROFILE === 'release') {
      const dist = new URL('../../../../../packages/rspack/dist/', import.meta.url);
      for (const file of readdirSync(dist, { recursive: true })) {
        if (!/\.(?:mjs|cjs|js)$/.test(file)) continue;
        const code = readFileSync(new URL(file, dist), 'utf8');
        assert.doesNotMatch(code, /node:string_decoder|stream-json|stream-chain|parseHeapSnapshot|readHeapGraph|RSPACK_ALLOCATIVE_DIR|__rspack_internal_allocative|__rspack_internal_runJsSnapshot|writeHeapSnapshot/, file);
      }
      assert(!existsSync(new URL('util/heapSnapshot.d.mts', dist)));
      const manifest = JSON.parse(readFileSync(new URL('../../../../../packages/rspack/package.json', import.meta.url), 'utf8'));
      for (const group of ['dependencies', 'devDependencies', 'optionalDependencies']) {
        assert(!manifest[group]?.['stream-json']);
        assert(!manifest[group]?.['stream-chain']);
      }
      return;
    }

    const directory = mkdtempSync(join(tmpdir(), 'rspack-object-markers-'));
    try {
      if (process.env.ALLOCATIVE) {
        execFileSync(process.execPath, ['--expose-gc', join(import.meta.dirname, 'capture-owned.cjs')], {
          cwd: import.meta.dirname,
          env: { ...process.env, RSPACK_ALLOCATIVE_DIR: join(directory, 'owned') },
          timeout: 60_000,
          stdio: 'pipe',
        });
      }
      const path = join(directory, 'objects.heapsnapshot');
      execFileSync(process.execPath, ['--expose-gc', join(import.meta.dirname, 'capture-markers.mjs')], {
        cwd: import.meta.dirname,
        env: {
          ...process.env,
          RSPACK_ALLOCATIVE_DIR: directory,
          ALLOCATIVE_TEST_SNAPSHOT: path,
        },
        timeout: 60_000,
      });
      if (process.env.ALLOCATIVE) {
        const profiles = readdirSync(directory).filter(name => name.endsWith('.allocative'));
        assert.deepEqual(profiles.sort(), ['0-build.allocative', '1-rebuild.allocative', '2-rebuild.allocative', '3-build.allocative']);
        for (const profile of profiles) {
          const native = readFileSync(join(directory, profile), 'utf8');
          assert.match(native, /rspack_binding_api::JsCompiler;compiler;/);
          assert(!native.split('\n').some(line => line.startsWith('rspack_core::compiler::Compiler;')));
          const name = profile.slice(0, -'.allocative'.length);
          const metadata = JSON.parse(readFileSync(join(directory, `${name}.metadata.json`), 'utf8'));
          assert.equal(metadata.snapshot, name);
          const total = native.trim().split('\n').reduce((sum, line) => sum + Number(line.slice(line.lastIndexOf(' ') + 1)), 0);
          assert.equal(metadata.represented_bytes, total);
          assert(metadata.ustr_arena_capacity_bytes >= metadata.ustr_arena_used_bytes);
          assert.match(native, /GlobalUstrStringArena;arena;/);
          assert(existsSync(join(directory, `${name}.warnings.txt`)));
          assert.doesNotMatch(readFileSync(join(directory, `${name}.warnings.txt`), 'utf8'), /Incorrect size declaration/);
          const counts = JSON.parse(readFileSync(join(directory, `${name}.counts.json`), 'utf8'));
          assert.equal(counts.snapshot, name);
          assert.equal(counts.unit, 'entries');
          for (const kind of ['modules', 'dependencies']) {
            const entries = counts.counts.filter(entry => entry.path[2] === kind);
            assert(entries.length > 0);
            assert(entries.reduce((sum, entry) => sum + entry.count, 0) > 0);
            for (const entry of entries) {
              assert.deepEqual(entry.path.slice(0, 2), ['Compiler', 'Compilation']);
              assert.equal(entry.basis, 'live entries in the current compilation module graph');
            }
          }
          const identities = JSON.parse(readFileSync(join(directory, `${name}.objects.json`), 'utf8'));
          const stats = identities.objects.filter(object => object.type_name.endsWith('::JsStats'));
          if (name === '2-rebuild' || name === '3-build') {
            assert.equal(stats.length, 0, 'released Stats wrappers survived GC');
            assert.doesNotMatch(native, /rspack_binding_api::stats::JsStats/);
            if (name === '2-rebuild') {
              assert.match(native, /NapiOwnerAllocations;.*;payload;rspack_core::stats::Stats(?:<[^;]*>)?;/,
                'owner-finalized Stats allocations disappeared with their wrappers');
            } else {
              assert.doesNotMatch(native, /rspack_core::stats::Stats(?:<[^;]*>)?;/,
                'finalized owner left a tracked Stats allocation');
            }
          } else {
            assert(stats.length >= (name === '0-build' ? 1 : 2));
            assert(stats.every(object => object.measured), 'live Stats wrapper was not measured');
            assert.match(native, /rspack_binding_api::stats::JsStats;inner;.*;payload;rspack_core::stats::Stats(?:<[^;]*>)?;/);
            assert(!native.split('\n').some(line => line.includes('::JsStats;') && line.includes(';rspack_core::compilation::Compilation;')),
              'Stats must not recount its borrowed Compilation');
          }
          const heap = JSON.parse(readFileSync(join(directory, 'v8', `${name}.heapsnapshot`), 'utf8'));
          const linked = convert(heap, identities).report;
          assert.equal(linked.unmatched_bindings, 0);
          assert.equal(linked.native_owners.length, 1);
          for (const kind of ['Compiler', 'Compilation', 'Cache']) {
            const root = linked.root_links.find(root => root.kind === kind && root.rust_target?.path?.length);
            assert(root?.binding_node_id, `missing ${kind} binding`);
            assert(root.rust_target?.path?.length, `missing ${kind} target`);
          }
          for (const type of ['JsCompiler', 'JsCompilation', 'JsResolverFactory', 'ModuleHandle']) {
            assert(identities.objects.some(object => object.type_name.endsWith(`::${type}`)), `missing ${type} registration`);
          }

        }
      }
      const bytes = readFileSync(path);
      let offset = 0;
      const snapshot = parseHeapSnapshot(() => {
        if (offset === bytes.length) return null;
        const chunk = bytes.subarray(offset, offset + 65537);
        offset += chunk.length;
        return chunk;
      });
      const { report, categoryFolded } = convert(snapshot, { objects: [] });
      const kinds = new Map(
        (report.marked_objects ?? []).map((item) => [item.kind, item]),
      );
      for (const kind of [
        'Asset', 'Assets', 'CompilationHooks', 'CompilerHooks',
        'Module', 'ModuleGraph', 'Stats',
      ]) {
        assert(kinds.get(kind)?.shallow_bytes > 0, `missing ${kind} marker`);
      }
      assert.match(categoryFolded.rspack, /;Asset;object: Object \d+\n/);
      assert.match(categoryFolded.rspack, /;Assets;object: Array \d+\n/);
      assert.equal(
        Object.values(report.category_bytes).reduce((a, b) => a + b, 0),
        report.represented_bytes,
      );

      const graph = readHeapGraph(snapshot);
      let pluginNode;
      for (let edge = 0; edge < snapshot.edges.length; edge += graph.edgeWidth) {
        if (
          graph.edgeType(edge) === 'property' &&
          graph.edgeName(edge) === 'rspackAllocativeTestSharedPayload'
        ) {
          pluginNode = graph.target(edge);
          break;
        }
      }
      assert.notEqual(pluginNode, undefined);
      const fields = snapshot.snapshot.meta.node_fields;
      snapshot.nodes[pluginNode * fields.length + fields.indexOf('self_size')]++;
      const changed = convert(snapshot, { objects: [] }).report.category_bytes;
      assert.deepEqual(changed, {
        ...report.category_bytes,
        shared_or_unknown: report.category_bytes.shared_or_unknown + 1,
      });
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  },
};
