import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {
  readHeapGraph,
  redactHeapSnapshot,
} from '../../../../../packages/rspack/src/util/heapSnapshot.mjs';
import { convert, main } from '../../../../../scripts/v8-to-allocative.mjs';

const foldedBytes = (folded) =>
  folded
    .trim()
    .split('\n')
    .reduce((sum, row) => sum + Number(row.split(' ').at(-1)), 0);

const assertRootTargets = (directory, name = '0-build') => {
  const prefix = path.join(directory, 'reports', name);
  const report = JSON.parse(fs.readFileSync(`${prefix}.links.json`, 'utf8'));
  const combined = fs.readFileSync(`${prefix}.combined.allocative`, 'utf8');
  const rust = fs.readFileSync(
    path.join(directory, `${name}.allocative`),
    'utf8',
  );
  const wrapperType = 'rspack_binding_api::JsCompiler';
  const coreType = 'rspack_core::compiler::Compiler';
  const compiler = report.root_links.find(
    (root) => root.kind === 'Compiler' && root.rust_target,
  );
  assert(compiler, 'the active public Compiler must identify its native owner');
  assert.equal(compiler.rust_target.type_name, coreType);
  assert.equal(
    compiler.rust_target.path[0],
    wrapperType,
    'capture must start at the real Rust JsCompiler, including its fields',
  );
  const binding = report.links.find(
    (link) => link.js_node_id === compiler.binding_node_id,
  ).rust_objects[0];
  assert.equal(binding.measured, true);
  assert.deepEqual(binding.path, [wrapperType]);
  const owner = ['JavaScript', 'Rspack', 'Compiler', wrapperType].join(';');
  assert(combined.includes(`${owner};compiler;`));
  assert(combined.includes(`${compiler.rust_target.path.slice(1).join(';')};`));
  // All wrapper fields must be represented, even opaque handles and empty maps.
  for (const field of [
    'unsafe_fast_drop',
    'file_system_dependency_string_pool',
    'js_helpers',
    'infrastructure_log_dispatcher',
    'compiler_scoped_tsfn_manager',
    'js_hooks_plugin',
    'compiler',
    'state',
    'include_dependencies_map',
    'entry_dependencies_map',
    'compiler_context',
    'virtual_file_store',
  ]) {
    assert(
      rust.includes(`${wrapperType};${field}`),
      `missing wrapper field: ${field}`,
    );
    assert(
      combined.includes(`${owner};${field}`),
      `missing owned wrapper field: ${field}`,
    );
  }
  assert(
    !rust.split('\n').some((line) => line.startsWith(`${coreType};`)),
    'core Compiler must not also be traversed as a separate root',
  );
  for (const line of rust
    .split('\n')
    .filter(
      (line) =>
        line.startsWith(`${wrapperType};`) ||
        line.startsWith(`${wrapperType} `),
    )) {
    assert.equal(
      combined
        .split('\n')
        .filter((row) => row === `JavaScript;Rspack;Compiler;${line}`).length,
      1,
      'each native sample must move to the JS owner exactly once',
    );
  }
  assert(
    !combined.split('\n').some((line) => line.startsWith(`${wrapperType};`)),
    'the native owner must not be duplicated outside its JS Compiler',
  );
  const warnings = fs.readFileSync(
    path.join(directory, `${name}.warnings.txt`),
    'utf8',
  );
  assert(!warnings.includes('Incorrect size declaration'), warnings);
  for (const root of report.root_links.filter((root) => root.rust_target)) {
    const binding = report.links.find(
      (link) => link.js_node_id === root.binding_node_id,
    );
    assert.deepEqual(root.rust_target, binding.rust_objects[0].target);
    assert.equal(root.rust_target.path[0], wrapperType);
    assert(root.rust_target.type_name.endsWith(`::${root.kind}`));
  }
  assert.equal(
    foldedBytes(combined),
    foldedBytes(rust) + report.represented_bytes,
  );
};

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
      const secret = 'rspack-test-private-heap-content';
      const env = {
        ...process.env,
        RSPACK_ALLOCATIVE_DIR: directory,
        ALLOCATIVE_TEST_CONTEXT: temporary,
        ALLOCATIVE_TEST_CONTENT: secret,
      };
      delete env.RSPACK_ALLOCATIVE_INCLUDE_CONTENT;
      const fixture = path.join(import.meta.dirname, 'capture.cjs');
      const child = spawnSync(process.execPath, ['--expose-gc', fixture], {
        env,
        encoding: 'utf8',
        timeout: 60000,
      });
      assert.ifError(child.error);
      assert.equal(child.status, 0, child.stderr);
      const captured = JSON.parse(child.stdout.trim().split('\n').at(-1));
      assert.equal(
        captured.enabled,
        fs.existsSync(path.join(directory, '0-build.allocative')),
        'a Rust snapshot must include live binding identities',
      );
      const disabledEnv = {
        ...process.env,
        ALLOCATIVE_TEST_CONTEXT: temporary,
      };
      delete disabledEnv.RSPACK_ALLOCATIVE_DIR;
      const disabled = spawnSync(process.execPath, ['--expose-gc', fixture], {
        env: disabledEnv,
        encoding: 'utf8',
        timeout: 60000,
      });
      assert.ifError(disabled.error);
      assert.equal(disabled.status, 0, disabled.stderr);
      assert.equal(
        JSON.parse(disabled.stdout.trim().split('\n').at(-1)).enabled,
        false,
      );
      if (captured.enabled) {
        const rustOnlyDirectory = path.join(temporary, 'rust-only');
        const rustOnly = spawnSync(process.execPath, ['--expose-gc', fixture], {
          env: {
            ...env,
            RSPACK_ALLOCATIVE_DIR: rustOnlyDirectory,
            ALLOCATIVE_TEST_RUST_ONLY: '1',
          },
          encoding: 'utf8',
          timeout: 60000,
        });
        assert.ifError(rustOnly.error);
        assert.equal(rustOnly.status, 0, rustOnly.stderr);
        assert(
          fs
            .readFileSync(
              path.join(rustOnlyDirectory, '0-build.allocative'),
              'utf8',
            )
            .includes('rspack_binding_api::JsCompiler;compiler;'),
        );
        assert.equal(fs.existsSync(path.join(rustOnlyDirectory, 'v8')), false);
        const prefix = path.join(directory, '0-build');
        const heapPath = path.join(directory, 'v8', '0-build.heapsnapshot');
        const heap = fs.readFileSync(heapPath, 'utf8');
        assert.equal(fs.existsSync(`${prefix}.heapsnapshot`), false);
        assert(!heap.includes(secret));
        const snapshot = JSON.parse(heap);
        assert.deepEqual(snapshot.strings, ['']);
        assert(snapshot.rspack_v8_source_nodes.length > 0);
        assert.deepEqual(
          new Set(snapshot.rspack_js_roots.map(([, kind]) => kind)),
          new Set(['Compiler', 'Compilation', 'Cache']),
        );
        const objects = JSON.parse(
          fs.readFileSync(`${prefix}.objects.json`, 'utf8'),
        );
        assert(
          objects.objects.every((object) => !Object.hasOwn(object, 'address')),
        );
        assert.throws(() => main(['--include-content', heapPath]), /redacted/);
        main([heapPath]);
        const reports = path.join(directory, 'reports', '0-build');
        const links = JSON.parse(
          fs.readFileSync(`${reports}.links.json`, 'utf8'),
        );
        assertRootTargets(directory);
        assert.equal(links.unmatched_bindings, 0);
        let categorizedBytes = 0;
        for (const category of ['v8_code', 'rspack', 'shared_or_unknown']) {
          const folded = fs.readFileSync(
            `${reports}.js.${category}.allocative`,
            'utf8',
          );
          assert(!folded.includes(secret));
          assert.equal(foldedBytes(folded), links.category_bytes[category]);
          assert(links.category_bytes[category] > 0);
          categorizedBytes += links.category_bytes[category];
        }
        assert.equal(categorizedBytes, links.represented_bytes);
        assert(
          links.matched_bindings >=
            new Set(captured.seen.map(({ pointer }) => pointer)).size,
        );
        for (const link of links.links) {
          assert.equal(link.rust_objects.length, 1);
          assert.equal(link.js_name, undefined);
          assert.equal(link.rust_address, undefined);
        }
        for (const suffix of [
          '.js.allocative',
          '.combined.allocative',
          '.links.json',
        ]) {
          assert(
            !fs.readFileSync(`${reports}${suffix}`, 'utf8').includes(secret),
          );
        }

        const fullDirectory = path.join(temporary, 'full-capture');
        const full = spawnSync(process.execPath, ['--expose-gc', fixture], {
          env: {
            ...env,
            RSPACK_ALLOCATIVE_DIR: fullDirectory,
            RSPACK_ALLOCATIVE_INCLUDE_CONTENT: '1',
          },
          encoding: 'utf8',
          timeout: 60000,
        });
        assert.ifError(full.error);
        assert.equal(full.status, 0, full.stderr);
        const fullCapture = JSON.parse(full.stdout.trim().split('\n').at(-1));
        const fullHeap = path.join(fullDirectory, 'v8', '0-build.heapsnapshot');
        assert(fs.readFileSync(fullHeap, 'utf8').includes(secret));
        main([fullHeap]);
        assertRootTargets(fullDirectory);
        assert(
          !fs
            .readFileSync(
              path.join(fullDirectory, 'reports', '0-build.js.allocative'),
              'utf8',
            )
            .includes(secret),
        );
        const fullReports = fs.mkdtempSync(
          path.join(temporary, 'full-reports-'),
        );
        main(['--include-content', '--out-dir', fullReports, fullHeap]);
        assert(
          fs
            .readFileSync(
              path.join(fullReports, '0-build.js.allocative'),
              'utf8',
            )
            .includes(secret),
        );
        const fullLinks = JSON.parse(
          fs.readFileSync(path.join(fullReports, '0-build.links.json'), 'utf8'),
        );
        for (const { label, pointer } of fullCapture.seen) {
          assert.equal(
            fullLinks.links.filter((link) => link.rust_address === pointer)
              .length,
            1,
            label,
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

        const multipleDirectory = path.join(temporary, 'multiple');
        const multiple = spawnSync(
          process.execPath,
          ['--expose-gc', path.join(import.meta.dirname, 'targets.cjs')],
          {
            env: {
              ...env,
              RSPACK_ALLOCATIVE_DIR: multipleDirectory,
              RSPACK_ALLOCATIVE_INCLUDE_CONTENT: '1',
            },
            encoding: 'utf8',
            timeout: 60000,
          },
        );
        assert.ifError(multiple.error);
        assert.equal(multiple.status, 0, multiple.stderr);
        for (const [name, active] of [
          ['0-build', 'first'],
          ['1-build', 'second'],
          ['2-rebuild', 'first'],
        ]) {
          const heap = JSON.parse(
            fs.readFileSync(
              path.join(multipleDirectory, 'v8', `${name}.heapsnapshot`),
              'utf8',
            ),
          );
          const identities = JSON.parse(
            fs.readFileSync(
              path.join(multipleDirectory, `${name}.objects.json`),
              'utf8',
            ),
          );
          const graph = readHeapGraph(heap);
          const compilerNames = new Map();
          for (let i = 0; i < graph.count; i++) {
            for (
              let e = graph.offsets[i];
              e < graph.offsets[i + 1];
              e += graph.edgeWidth
            ) {
              if (
                graph.edgeType(e) === 'property' &&
                graph.edgeName(e) === 'name'
              ) {
                compilerNames.set(
                  graph.nodeId(i),
                  graph.nodeName(graph.target(e)),
                );
              }
            }
          }
          main([path.join(multipleDirectory, 'v8', `${name}.heapsnapshot`)]);
          assertRootTargets(multipleDirectory, name);
          const raw = convert(heap, identities);
          const compilerLinks = raw.report.root_links.filter(
            (link) => link.kind === 'Compiler',
          );
          assert.equal(compilerLinks.length, 2);
          for (const link of compilerLinks) {
            assert.equal(
              Boolean(link.rust_target),
              compilerNames.get(link.js_node_id) === active,
              `${name}: only the active compiler may match its Rust target`,
            );
          }
          assert.equal(raw.report.matched_roots, name === '2-rebuild' ? 4 : 3);
          const { redacted, markers } = redactHeapSnapshot(heap);
          const nodes = new Map(
            markers.map(([id, address, type]) => [`${address}:${type}`, id]),
          );
          const sanitized = convert(redacted, {
            ...identities,
            objects: identities.objects.map(({ address, ...object }) => ({
              ...object,
              js_node_id: nodes.get(`${address}:${object.type_name}`),
            })),
          });
          assert.deepEqual(sanitized.report.root_links, raw.report.root_links);
          assert.equal(
            foldedBytes(sanitized.combinedFolded),
            raw.report.represented_bytes,
          );
        }
      } else {
        assert.equal(fs.existsSync(path.join(directory, 'v8')), false);
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
    assert.match(result.combinedFolded, /Bindings;Binding;JavaScript/);
    assert.equal(result.report.category_bytes.rspack, 64);
    assert.doesNotMatch(result.folded, /Wrapper|child|0x1234/);
    assert.doesNotMatch(JSON.stringify(result.report), /Wrapper|0x1234/);
    assert.match(
      convert(snapshot, identities, { includeContent: true }).folded,
      /child%3Bname/,
    );
    assert.equal(foldedBytes(result.folded), 128);
    assert.equal(
      convert(snapshot, { snapshot: '0-build', objects: [] }).report
        .unmatched_bindings,
      1,
    );
    const unrelatedStrings = [...strings];
    unrelatedStrings[2] = 'not-a-pointer';
    const unrelated = convert(
      { ...snapshot, strings: unrelatedStrings },
      identities,
    );
    assert.equal(unrelated.report.links.length, 0);
    assert.equal(unrelated.report.represented_bytes, 128);
    assert.equal(
      convert(snapshot, identities, { maxDepth: 1 }).report.represented_bytes,
      128,
    );
    assert.throws(
      () => convert({ ...snapshot, edges: [0, 7, 3] }, identities),
      /edge counts/,
    );

    const ownershipHeap = {
      snapshot: {
        meta: {
          ...meta,
          node_types: [['synthetic', 'object', 'string', 'code']],
        },
      },
      strings: [
        'root',
        'compiler',
        'script',
        'other',
        'shared',
        'exclusive',
        'private source text',
        'Compiler',
        '__rspack_internal_allocativeRoot',
        'source',
        'ref',
      ],
      nodes: [
        [0, 0, 1, 0, 3], // Global root reaches Compiler, Script, and an external owner.
        [1, 1, 3, 10, 3], // Compiler owns the marker and references both payloads.
        [3, 2, 5, 20, 1], // Script has a source string.
        [1, 3, 7, 5, 2], // External owner shares one payload and weakly references the other.
        [1, 4, 9, 40, 0],
        [1, 5, 11, 30, 0],
        [2, 6, 13, 60, 0],
        [2, 7, 15, 1, 0],
      ].flat(),
      edges: [
        [0, 10, 1],
        [0, 10, 2],
        [0, 10, 3],
        [0, 10, 4],
        [0, 10, 5],
        [0, 8, 7],
        [1, 9, 6],
        [0, 10, 4],
        [2, 10, 5],
      ].flatMap(([type, name, target]) => [type, name, target * 5]),
    };
    const emptyIdentities = { snapshot: '0-build', objects: [] };
    const raw = convert(ownershipHeap, emptyIdentities);
    assert.deepEqual(raw.report.category_bytes, {
      v8_code: 80,
      rspack: 41,
      shared_or_unknown: 45,
    });
    const { redacted } = redactHeapSnapshot(ownershipHeap);
    assert.deepEqual(redacted.rspack_v8_source_nodes, [13]);
    assert.deepEqual(redacted.rspack_js_roots, [[3, 'Compiler']]);
    const sanitized = convert(redacted, emptyIdentities);
    assert.deepEqual(
      sanitized.report.category_bytes,
      raw.report.category_bytes,
    );
    assert.equal(foldedBytes(sanitized.folded), 166);
    assert.doesNotMatch(sanitized.folded, /private source text/);
  },
};
