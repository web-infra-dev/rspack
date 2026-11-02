import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import { convert } from '../../../../../scripts/v8-to-allocative.mjs';

export default {
  afterExecute() {
    const temporary = fs.mkdtempSync(
      path.join(os.tmpdir(), 'rspack-allocative-test-'),
    );
    try {
      fs.writeFileSync(
        path.join(temporary, 'index.js'),
        "import('./async.js');",
      );
      fs.writeFileSync(
        path.join(temporary, 'async.js'),
        "export default 'async';",
      );
      const directory = path.join(temporary, 'capture');
      const env = {
        ...process.env,
        RSPACK_ALLOCATIVE_DIR: directory,
        ALLOCATIVE_TEST_CONTEXT: temporary,
      };
      const fixture = path.join(import.meta.dirname, 'capture.cjs');
      const child = spawnSync(process.execPath, ['--expose-gc', fixture], {
        env,
        encoding: 'utf8',
        timeout: 60000,
      });
      assert.ifError(child.error);
      assert.equal(child.status, 0, child.stderr);
      const captured = JSON.parse(child.stdout.trim().split('\n').at(-1));
      if (captured.enabled) {
        const prefix = path.join(directory, '0-build');
        const result = convert(
          JSON.parse(fs.readFileSync(`${prefix}.heapsnapshot`, 'utf8')),
          JSON.parse(fs.readFileSync(`${prefix}.objects.json`, 'utf8')),
        );
        assert.equal(result.report.unmatched_bindings, 0);
        for (const object of captured.seen) {
          assert(
            result.report.links.some(
              (link) => link.rust_address === object.pointer,
            ),
            object.type,
          );
        }
        const missingGc = spawnSync(process.execPath, [fixture], {
          env: { ...env, RSPACK_ALLOCATIVE_DIR: path.join(temporary, 'no-gc') },
          encoding: 'utf8',
          timeout: 60000,
        });
        assert.ifError(missingGc.error);
        assert.notEqual(missingGc.status, 0);
        assert.match(missingGc.stderr, /--expose-gc/);
      } else {
        assert.equal(
          fs.existsSync(path.join(directory, '0-build.heapsnapshot')),
          false,
        );
      }
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
    const strings = [
      'root',
      'Wrapper',
      '0x1234',
      'Binding',
      'child;name',
      '__rspackRustPtr',
      '__rspackRustType',
      'owned',
      'cycle',
      'weak',
    ];
    const meta = {
      node_fields: ['type', 'name', 'id', 'self_size', 'edge_count'],
      node_types: [['synthetic', 'object', 'string']],
      edge_fields: ['type', 'name_or_index', 'to_node'],
      edge_types: [['property', 'internal', 'weak']],
    };
    const snapshot = {
      snapshot: { meta },
      strings,
      // Root -> wrapper -> payload -> wrapper cycle. One weak-only payload.
      nodes: [
        0, 0, 1, 0, 1, 1, 1, 3, 16, 3, 2, 2, 5, 8, 0, 2, 3, 7, 8, 0, 1, 4, 9,
        32, 2, 1, 4, 11, 64, 0,
      ],
      edges: [0, 7, 5, 0, 5, 10, 0, 6, 15, 0, 7, 20, 0, 8, 5, 2, 9, 25],
    };
    const identities = {
      snapshot: '0-build',
      objects: [
        {
          address: '0x1234',
          type_name: 'Binding',
          path: ['Bindings', 'Binding'],
        },
      ],
    };
    const result = convert(snapshot, identities);
    assert.equal(result.report.represented_bytes, 128);
    assert.equal(result.report.unreachable_bytes, 64);
    assert.equal(result.report.matched_bindings, 1);
    assert.equal(result.report.links[0].js_node_id, 3);
    assert.match(result.folded, /Bindings;Binding;JavaScript/);
    assert.match(result.folded, /child%3Bname/);
    assert.equal(
      result.folded
        .trim()
        .split('\n')
        .reduce((sum, row) => sum + Number(row.split(' ').at(-1)), 0),
      128,
    );
    assert.equal(
      convert(snapshot, { snapshot: '0-build', objects: [] }).report
        .unmatched_bindings,
      1,
    );
    assert.equal(
      convert(snapshot, identities, 1).report.represented_bytes,
      128,
    );
    assert.throws(
      () => convert({ ...snapshot, edges: [0, 7, 3] }, identities),
      /edge counts/,
    );
  },
};
