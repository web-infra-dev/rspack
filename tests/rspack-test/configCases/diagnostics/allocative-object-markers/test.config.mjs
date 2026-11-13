import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { convert } from '../../../../../.agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs';
import { readHeapGraph } from '../../../../../packages/rspack/src/util/heapSnapshot.mjs';

export default {
  afterExecute() {
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
      const snapshot = JSON.parse(readFileSync(path, 'utf8'));
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
