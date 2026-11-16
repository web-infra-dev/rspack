import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { convert } from '../../../../../.agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs';
import { parseHeapSnapshot, readHeapGraph } from '../../../../../.agents/skills/rspack-allocative/scripts/heapSnapshot.mjs';

export default {
  afterExecute() {
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
      const path = join(directory, 'objects.heapsnapshot');
      execFileSync(process.execPath, ['--expose-gc', '--input-type=module', '-e', `
        import assert from 'node:assert/strict';
        import { writeHeapSnapshot } from 'node:v8';
        import { rspack } from '@rspack/core';
        const compiler = rspack({
          mode: 'development',
          context: process.cwd(),
          entry: './index.js',
          output: { path: process.env.RSPACK_ALLOCATIVE_DIR },
        });
        compiler.hooks.afterCompile.tap('AllocativeObjects', compilation => {
          const assets = compilation.getAssets();
          const asset = assets[0];
          assert.equal(Object.getPrototypeOf(asset), Object.prototype);
          const pluginObject = { pluginPayload: true };
          const marker = '__rspack_internal_allocativeObject';
          assert.equal(Object.getOwnPropertyDescriptor(asset, marker)?.enumerable, false);
          assert(!Object.hasOwn(pluginObject, marker));
          asset.rspackAllocativeTestSharedPayload = pluginObject;
          globalThis.allocativeObjects = {
            asset, assets, pluginObject, module: [...compilation.modules][0],
            stats: compilation.getStats(), moduleGraph: compilation.moduleGraph,
          };
          writeHeapSnapshot(process.env.ALLOCATIVE_TEST_SNAPSHOT);
        });
        try {
          await new Promise((resolve, reject) => compiler.run((error, stats) => {
            if (error) reject(error);
            else if (stats.hasErrors()) reject(new Error(stats.toString()));
            else resolve();
          }));
          if (process.env.ALLOCATIVE) {
            await new Promise((resolve, reject) => compiler.__internal__rebuild(
              new Set(), new Set(), error => error ? reject(error) : resolve(),
            ));
          }
        } finally {
          await new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
        }
      `], {
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
        assert.deepEqual(profiles.sort(), ['0-build.allocative', '1-rebuild.allocative']);
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
          const heap = JSON.parse(readFileSync(join(directory, 'v8', `${name}.heapsnapshot`), 'utf8'));
          const linked = convert(heap, identities).report;
          assert.equal(linked.unmatched_bindings, 0);
          assert.equal(linked.native_owners.length, 1);
          for (const kind of ['Compiler', 'Compilation', 'Cache']) {
            const root = linked.root_links.find(root => root.kind === kind);
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
      assert.match(categoryFolded.rspack, /;Asset \d+\n/);
      assert.match(categoryFolded.rspack, /;Assets \d+\n/);
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
