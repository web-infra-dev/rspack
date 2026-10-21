import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

// Match allocative's folded-stack escaping. Weights are shallow bytes, never
// retained sizes: shared objects and cycles must contribute only once.
export function escapeFrame(value) {
  return value.replace(
    /[%;\x00-\x1f\x7f]/g,
    (ch) => `%${ch.codePointAt(0).toString(16).toUpperCase().padStart(2, '0')}`,
  );
}

export function convert(snapshot, identities, maxDepth = 32) {
  const { meta } = snapshot.snapshot;
  const nodeFields = meta.node_fields;
  const edgeFields = meta.edge_fields;
  const nodeWidth = nodeFields.length;
  const edgeWidth = edgeFields.length;
  const { nodes, edges, strings } = snapshot;
  const field = (fields, name) => {
    const index = fields.indexOf(name);
    if (index < 0) throw new Error(`Missing V8 snapshot field: ${name}`);
    return index;
  };
  const nt = field(nodeFields, 'type');
  const nn = field(nodeFields, 'name');
  const ni = field(nodeFields, 'id');
  const ns = field(nodeFields, 'self_size');
  const nc = field(nodeFields, 'edge_count');
  const et = field(edgeFields, 'type');
  const en = field(edgeFields, 'name_or_index');
  const to = field(edgeFields, 'to_node');
  const nodeTypes = meta.node_types[nt];
  const edgeTypes = meta.edge_types[et];
  const count = nodes.length / nodeWidth;
  if (!Number.isInteger(count) || edges.length % edgeWidth !== 0 || !count) {
    throw new Error('Invalid V8 snapshot array lengths');
  }
  const offsets = new Float64Array(count + 1);
  for (let i = 0; i < count; i++) {
    offsets[i + 1] = offsets[i] + nodes[i * nodeWidth + nc] * edgeWidth;
  }
  if (offsets[count] !== edges.length)
    throw new Error('Invalid V8 edge counts');
  const target = (edge) => {
    const offset = edges[edge + to];
    if (offset % nodeWidth || offset < 0 || offset >= nodes.length) {
      throw new Error(`Invalid V8 edge target: ${offset}`);
    }
    return offset / nodeWidth;
  };
  const nodeName = (i) => strings[nodes[i * nodeWidth + nn]];
  const nodeType = (i) => nodeTypes[nodes[i * nodeWidth + nt]];
  const native = new Map();
  for (const object of identities.objects) {
    const key = `${object.address.toLowerCase()}:${object.type_name}`;
    const matches = native.get(key) ?? [];
    matches.push(object);
    native.set(key, matches);
  }
  const bindings = new Map();
  const links = [];
  for (let i = 0; i < count; i++) {
    let pointer;
    let type;
    for (let e = offsets[i]; e < offsets[i + 1]; e += edgeWidth) {
      if (edgeTypes[edges[e + et]] !== 'property') continue;
      const name = strings[edges[e + en]];
      if (name !== '__rspackRustPtr' && name !== '__rspackRustType') continue;
      const value = target(e);
      if (nodeType(value) !== 'string') continue;
      if (name === '__rspackRustPtr') pointer = nodeName(value);
      else type = nodeName(value);
    }
    if (!pointer) continue;
    if (!/^0x[0-9a-f]+$/i.test(pointer))
      throw new Error(`Invalid Rust pointer: ${pointer}`);
    const matches = native.get(`${pointer.toLowerCase()}:${type}`) ?? [];
    const link = {
      js_node_id: nodes[i * nodeWidth + ni],
      js_name: nodeName(i),
      rust_address: pointer,
      rust_type: type,
      rust_objects: matches,
    };
    links.push(link);
    if (matches.length) bindings.set(i, link);
  }

  // A first-strong-owner BFS spanning forest. This is an attribution policy,
  // not V8's dominator tree. Weak edges never establish ownership.
  const parents = new Int32Array(count).fill(-1);
  const queue = new Uint32Array(count);
  parents[0] = 0;
  queue[0] = 0;
  let end = 1;
  for (let start = 0; start < end; start++) {
    const i = queue[start];
    for (let e = offsets[i]; e < offsets[i + 1]; e += edgeWidth) {
      if (edgeTypes[edges[e + et]] === 'weak') continue;
      const child = target(e);
      if (parents[child] !== -1) continue;
      parents[child] = i;
      queue[end++] = child;
    }
  }
  const samples = new Map();
  let totalBytes = 0;
  let unreachableBytes = 0;
  for (let i = 0; i < count; i++) {
    const size = nodes[i * nodeWidth + ns];
    if (!Number.isSafeInteger(size) || size < 0)
      throw new Error('Invalid V8 self_size');
    totalBytes += size;
    if (!size) continue;
    if (parents[i] === -1) unreachableBytes += size;
    const frames = [];
    let cursor = i;
    let prefix = ['JavaScript'];
    let remaining = maxDepth;
    while (true) {
      const binding = bindings.get(cursor);
      if (binding) {
        prefix = [...binding.rust_objects[0].path, 'JavaScript'];
        frames.push(`${nodeType(cursor)}: ${nodeName(cursor)}`);
        break;
      }
      frames.push(`${nodeType(cursor)}: ${nodeName(cursor)}`);
      if (parents[cursor] === -1) {
        prefix.push('[unreachable by strong edges]');
        break;
      }
      if (cursor === 0) break;
      cursor = parents[cursor];
      if (--remaining === 0) {
        prefix.push('[path truncated]');
        break;
      }
    }
    const stack = [...prefix, ...frames.reverse()].map(escapeFrame).join(';');
    samples.set(stack, (samples.get(stack) ?? 0) + size);
  }
  return {
    folded: [...samples]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([stack, size]) => `${stack} ${size}\n`)
      .join(''),
    report: {
      snapshot: identities.snapshot,
      unit: 'bytes',
      attribution:
        'first strong owner by BFS; shallow self_size, not retained size; paths bounded by maxDepth',
      max_depth: maxDepth,
      represented_bytes: totalBytes,
      node_count: count,
      strong_reachable_nodes: end,
      unreachable_bytes: unreachableBytes,
      matched_bindings: links.filter((link) => link.rust_objects.length).length,
      unmatched_bindings: links.filter((link) => !link.rust_objects.length)
        .length,
      links,
    },
  };
}

export function main(args = process.argv.slice(2)) {
  const { values, positionals } = parseArgs({
    args,
    allowPositionals: true,
    options: {
      'out-dir': { type: 'string' },
      'max-depth': { type: 'string', default: '32' },
    },
  });
  if (positionals.length !== 1) {
    throw new Error(
      'Usage: node scripts/v8-to-allocative.mjs <prefix.heapsnapshot> [--out-dir <existing-dir>] [--max-depth 32]',
    );
  }
  const source = path.resolve(positionals[0]);
  const prefix = source.replace(/\.heapsnapshot$/, '');
  if (source === prefix) throw new Error('Input must end in .heapsnapshot');
  const maxDepth = Number(values['max-depth']);
  if (!Number.isSafeInteger(maxDepth) || maxDepth < 1)
    throw new Error('--max-depth must be a positive integer');
  const identities = JSON.parse(readFileSync(`${prefix}.objects.json`, 'utf8'));
  if (identities.snapshot !== path.basename(prefix))
    throw new Error('Rust snapshot identity does not match the V8 filename');
  const result = convert(
    JSON.parse(readFileSync(source, 'utf8')),
    identities,
    maxDepth,
  );
  const output = path.join(
    values['out-dir'] ?? path.dirname(prefix),
    path.basename(prefix),
  );
  const rust = readFileSync(`${prefix}.allocative`, 'utf8');
  // Refuse to overwrite captures or previous conversion results.
  writeFileSync(`${output}.js.allocative`, result.folded, { flag: 'wx' });
  writeFileSync(
    `${output}.combined.allocative`,
    rust + (rust.endsWith('\n') ? '' : '\n') + result.folded,
    { flag: 'wx' },
  );
  writeFileSync(
    `${output}.links.json`,
    `${JSON.stringify(result.report, null, 2)}\n`,
    { flag: 'wx' },
  );
  process.stdout.write(
    `${JSON.stringify({ snapshot: result.report.snapshot, represented_js_bytes: result.report.represented_bytes, matched_bindings: result.report.matched_bindings, unmatched_bindings: result.report.unmatched_bindings })}\n`,
  );
}

if (
  process.argv[1] &&
  pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url
) {
  main();
}
