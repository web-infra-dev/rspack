import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  readSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import {
  isRspackRootKind,
  parseHeapSnapshot,
  readBindingMarker,
  readHeapCategories,
  readHeapGraph,
} from './heapSnapshot.mjs';

// Match allocative's folded-stack escaping. Weights are shallow bytes, never
// retained sizes: shared objects and cycles must contribute only once.
export function escapeFrame(value) {
  return value.replace(
    /[%;\x00-\x1f\x7f]/g,
    (ch) => `%${ch.codePointAt(0).toString(16).toUpperCase().padStart(2, '0')}`,
  );
}

const categoryLabels = {
  rspack: 'Rspack',
  shared_or_unknown: 'Shared or unknown',
};

function strongPaths(graph, blocked = new Set()) {
  const { count, offsets, edgeWidth, target, edgeType } = graph;
  const parents = new Int32Array(count).fill(-1);
  const queue = new Uint32Array(count);
  parents[0] = 0;
  queue[0] = 0;
  let reachable = 1;
  for (let start = 0; start < reachable; start++) {
    const i = queue[start];
    for (let edge = offsets[i]; edge < offsets[i + 1]; edge += edgeWidth) {
      if (edgeType(edge) === 'weak') continue;
      const child = target(edge);
      if (parents[child] !== -1 || blocked.has(child)) continue;
      parents[child] = i;
      queue[reachable++] = child;
    }
  }
  return { parents, reachable };
}

function fold(samples) {
  return [...samples]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([stack, size]) => `${stack} ${size}\n`)
    .join('');
}

// Constructor names recognized as JavaScript/Node built-ins are always kept.
const builtinClassNames = new Set([
  'Object',
  'Array',
  'Function',
  'String',
  'Number',
  'Boolean',
  'BigInt',
  'Symbol',
  'Map',
  'Set',
  'WeakMap',
  'WeakSet',
  'WeakRef',
  'FinalizationRegistry',
  'Promise',
  'Date',
  'RegExp',
  'Error',
  'EvalError',
  'RangeError',
  'ReferenceError',
  'SyntaxError',
  'TypeError',
  'URIError',
  'AggregateError',
  'DOMException',
  'ArrayBuffer',
  'SharedArrayBuffer',
  'DataView',
  'Buffer',
  'Int8Array',
  'Uint8Array',
  'Uint8ClampedArray',
  'Int16Array',
  'Uint16Array',
  'Int32Array',
  'Uint32Array',
  'Float16Array',
  'Float32Array',
  'Float64Array',
  'BigInt64Array',
  'BigUint64Array',
  'URL',
  'URLSearchParams',
  'TextEncoder',
  'TextDecoder',
  'AbortController',
  'AbortSignal',
  'Event',
  'EventTarget',
  'Iterator',
  'AsyncIterator',
  'AsyncFunction',
  'GeneratorFunction',
  'AsyncGeneratorFunction',
  'Generator',
  'AsyncGenerator',
  'Arguments',
  'Math',
  'JSON',
  'Reflect',
  'Atomics',
  'WebAssembly.Module',
  'WebAssembly.Instance',
  'WebAssembly.Memory',
  'WebAssembly.Table',
  'WebAssembly.Global',
  'WebAssembly.Tag',
  'WebAssembly.Exception',
  'Array Iterator',
  'String Iterator',
  'Map Iterator',
  'Set Iterator',
  'Collator',
  'DateTimeFormat',
  'NumberFormat',
  'PluralRules',
  'RelativeTimeFormat',
  'ListFormat',
  'Locale',
  'DisplayNames',
  'Segmenter',
]);

function isBuiltinName(name) {
  return (
    builtinClassNames.has(name) ||
    name.startsWith('system / ') ||
    name.startsWith('Node / ')
  );
}

function isPrimitive(type) {
  return (
    type === 'string' ||
    type === 'concatenated string' ||
    type === 'sliced string' ||
    type === 'number' ||
    type === 'bigint' ||
    type === 'symbol'
  );
}

// These are storage for JS values, not engine metadata. Keep their weights
// without exposing V8's implementation names as ownership frames.
const storageNames = new Map([
  ['system / PropertyArray', 'object properties'],
  ['system / PropertyCell', 'property value'],
  ['system / Cell', 'captured value'],
  ['system / Context', 'closure context'],
  ['system / JSArrayBufferData', 'ArrayBuffer data'],
]);

function isV8Internal(type, name) {
  if (type === 'code' || type === 'synthetic' || type === 'object shape')
    return true;
  if (storageNames.has(name)) return false;
  return (
    type === 'hidden' || (type === 'object' && name.startsWith('system / '))
  );
}

export function convert(
  snapshot,
  identities,
  { maxDepth = 32, clearClassNames = false } = {},
) {
  const redacted = Array.isArray(snapshot.rspack_binding_nodes);
  const graph = readHeapGraph(snapshot);
  const { count, nodeType, nodeName, nodeSize, nodeId } = graph;
  const customClasses = new Set();
  if (clearClassNames) {
    for (let i = 0; i < count; i++) {
      if (
        nodeType(i) === 'object' &&
        nodeName(i) &&
        !isBuiltinName(nodeName(i))
      ) {
        customClasses.add(nodeName(i));
      }
    }
  }
  const displayName = (i) => {
    const type = nodeType(i);
    if (isPrimitive(type)) return undefined;
    const name = nodeName(i);
    if (!name || (clearClassNames && customClasses.has(name))) return undefined;
    return name;
  };
  const { sourceNodeIds, rspackRoots, rspackBindings, rspackObjects } = redacted
    ? {
        sourceNodeIds: snapshot.rspack_v8_source_nodes ?? [],
        rspackRoots: snapshot.rspack_js_roots ?? [],
        rspackBindings: snapshot.rspack_js_bindings ?? [],
        rspackObjects: new Map(),
      }
    : readHeapCategories(graph);
  const v8Sources = new Set(sourceNodeIds);
  const excluded = new Uint8Array(count);
  for (let i = 0; i < count; i++) {
    excluded[i] =
      v8Sources.has(nodeId(i)) || isV8Internal(nodeType(i), nodeName(i));
  }
  const jsRoots = new Map(
    rspackRoots.filter(([, kind]) => isRspackRootKind(kind)),
  );
  const bindingNodes = new Set(snapshot.rspack_binding_nodes);
  const native = new Map();
  for (const object of identities.objects) {
    const key = redacted
      ? object.js_node_id
      : object.address && `${object.address.toLowerCase()}:${object.type_name}`;
    if (key === undefined) continue;
    const matches = native.get(key) ?? [];
    matches.push(object);
    native.set(key, matches);
  }
  const bindings = new Map();
  const rspackAnchors = new Set();
  const links = [];
  for (let i = 0; i < count; i++) {
    const id = nodeId(i);
    if (jsRoots.has(id)) rspackAnchors.add(i);
    const marker = redacted ? undefined : readBindingMarker(graph, i);
    if (!bindingNodes.has(id) && !marker) continue;
    const key = marker ? `${marker[1]}:${marker[2]}` : id;
    const matches = native.get(key) ?? [];
    const link = {
      js_node_id: id,
      node_type: nodeType(i),
      self_size: nodeSize(i),
      rust_objects: matches.map(({ type_name, path, measured, target }) => ({
        type_name,
        path,
        measured,
        ...(target ? { target } : {}),
      })),
    };
    const name = displayName(i);
    if (name) link.js_name = name;
    links.push(link);
    if (matches.length) {
      bindings.set(i, link);
      rspackAnchors.add(i);
    }
  }

  const linksByNode = new Map(links.map((link) => [link.js_node_id, link]));
  const publicBindings = new Map(rspackBindings);
  const rootLinks = [...jsRoots].map(([js_node_id, kind]) => {
    const binding_node_id = publicBindings.get(js_node_id);
    const binding = linksByNode.get(binding_node_id);
    const rust_target = binding?.rust_objects[0]?.target;
    return {
      js_node_id,
      kind,
      binding_node_id,
      ...(rust_target ? { rust_target } : {}),
    };
  });
  const rootTargets = new Map(
    rootLinks
      .filter((link) => link.rust_target)
      .map((link) => [link.js_node_id, link.rust_target]),
  );

  // Only a measured owning wrapper can parent native bytes. Compilation views
  // and shared cache links remain metadata; they must not copy native subtrees.
  const nativeOwners = rootLinks.flatMap((root) => {
    const wrapper = linksByNode.get(root.binding_node_id)?.rust_objects[0];
    if (
      root.kind !== 'Compiler' ||
      !wrapper?.measured ||
      wrapper.path.length !== 1
    )
      return [];
    return [
      {
        js_node_id: root.js_node_id,
        binding_node_id: root.binding_node_id,
        rust_path: wrapper.path,
        js_path: ['JavaScript', 'Rspack', 'Compiler'],
      },
    ];
  });
  const owningRoots = new Set(nativeOwners.map((owner) => owner.js_node_id));
  const ownedPath = (path) => {
    const owner = nativeOwners.find((owner) =>
      owner.rust_path.every((frame, i) => path[i] === frame),
    );
    return owner ? [...owner.js_path, ...path] : path;
  };

  // The first pass supplies representative paths. A node is exclusive to
  // Rspack when removing all Rspack anchors cuts every strong path to it.
  const { parents, reachable } = strongPaths(graph);
  const outside = rspackAnchors.size
    ? strongPaths(graph, rspackAnchors).parents
    : parents;
  const samples = {
    rspack: new Map(),
    shared_or_unknown: new Map(),
  };
  const categoryBytes = { rspack: 0, shared_or_unknown: 0 };
  const combinedRspack = new Map();
  const markedObjects = new Map();
  let totalBytes = 0;
  let snapshotBytes = 0;
  let excludedBytes = 0;
  let excludedNodes = 0;
  let unreachableBytes = 0;
  for (let i = 0; i < count; i++) {
    const size = nodeSize(i);
    if (!Number.isSafeInteger(size) || size < 0)
      throw new Error('Invalid V8 self_size');
    snapshotBytes += size;
    if (excluded[i]) {
      excludedBytes += size;
      excludedNodes++;
      continue;
    }
    totalBytes += size;
    const id = nodeId(i);
    const kind = rspackObjects.get(id);
    if (kind) {
      const entry = markedObjects.get(kind) ?? {
        kind,
        object_count: 0,
        shallow_bytes: 0,
      };
      entry.object_count++;
      entry.shallow_bytes += size;
      markedObjects.set(kind, entry);
    }
    if (!size) continue;
    if (parents[i] === -1) unreachableBytes += size;
    const category =
      kind || rspackAnchors.has(i) || (parents[i] !== -1 && outside[i] === -1)
        ? 'rspack'
        : 'shared_or_unknown';
    categoryBytes[category] += size;
    const frames = [];
    let cursor = i;
    const prefix = ['JavaScript', categoryLabels[category]];
    let nativePath;
    let remaining = maxDepth;
    while (true) {
      const type = nodeType(cursor);
      const id = nodeId(cursor);
      const jsRoot = category === 'rspack' ? jsRoots.get(id) : undefined;
      if (!excluded[cursor]) {
        if (remaining-- === 0) {
          prefix.push('[path truncated]');
          break;
        }
        const name = displayName(cursor);
        const storage = storageNames.get(nodeName(cursor));
        frames.push(
          storage
            ? `storage: ${storage}`
            : name
              ? `${type}: ${name}`
              : (jsRoot ?? rspackObjects.get(id) ?? type),
        );
        const markedKind = rspackObjects.get(id);
        if (markedKind && name && markedKind !== name) frames.push(markedKind);
      }
      if (category === 'rspack') {
        const binding = bindings.get(cursor);
        if (binding) {
          const wrapper = binding.rust_objects[0];
          nativePath = wrapper.measured
            ? wrapper.path
            : (wrapper.target?.path ?? wrapper.path);
          break;
        }
        if (jsRoot) {
          if (!owningRoots.has(nodeId(cursor)))
            nativePath = rootTargets.get(nodeId(cursor))?.path;
          break;
        }
      }
      if (parents[cursor] === -1) {
        prefix.push('[unreachable by strong edges]');
        break;
      }
      if (cursor === 0) break;
      cursor = parents[cursor];
    }
    const stack = [...prefix, ...frames.reverse()].map(escapeFrame).join(';');
    const categorySamples = samples[category];
    categorySamples.set(stack, (categorySamples.get(stack) ?? 0) + size);
    if (category === 'rspack') {
      const combinedStack = nativePath
        ? `${ownedPath(nativePath).map(escapeFrame).join(';')};${stack}`
        : stack;
      combinedRspack.set(
        combinedStack,
        (combinedRspack.get(combinedStack) ?? 0) + size,
      );
    }
  }
  const categoryFolded = {
    rspack: fold(samples.rspack),
    shared_or_unknown: fold(samples.shared_or_unknown),
  };
  return {
    folded: Object.values(categoryFolded).join(''),
    combinedFolded: fold(combinedRspack) + categoryFolded.shared_or_unknown,
    categoryFolded,
    report: {
      snapshot: identities.snapshot,
      unit: 'bytes',
      includes_content: false,
      clear_class_names: clearClassNames,
      string_contents: 'omitted',
      ...(snapshot.rspack_string_contents
        ? { string_storage: snapshot.rspack_string_contents }
        : {}),
      ...(snapshot.rspack_string_preview
        ? { string_preview: snapshot.rspack_string_preview }
        : {}),
      attribution:
        'V8 code, script source and engine metadata excluded; marked Rspack objects and roots, plus nodes reachable only through Rspack roots or matched bindings; all other included nodes shared or unknown; shallow self_size once per node',
      max_depth: maxDepth,
      represented_bytes: totalBytes,
      snapshot_bytes: snapshotBytes,
      excluded_v8_bytes: excludedBytes,
      excluded_v8_nodes: excludedNodes,
      node_count: count,
      strong_reachable_nodes: reachable,
      unreachable_bytes: unreachableBytes,
      category_bytes: categoryBytes,
      marked_objects: [...markedObjects.values()].sort((a, b) =>
        a.kind.localeCompare(b.kind),
      ),
      matched_bindings: links.filter((link) => link.rust_objects.length).length,
      unmatched_bindings: links.filter((link) => !link.rust_objects.length)
        .length,
      matched_roots: rootTargets.size,
      unmatched_roots: rootLinks.length - rootTargets.size,
      root_links: rootLinks,
      native_owners: nativeOwners,
      links,
    },
  };
}

// Move each native sample once; never copy a target for every referring JS view.
export function combineRust(rust, result) {
  const owners = result.report.native_owners.map((owner) => ({
    native: owner.rust_path.map(escapeFrame).join(';'),
    prefix: owner.js_path.map(escapeFrame).join(';'),
  }));
  const moved = rust
    .split('\n')
    .filter(Boolean)
    .map((line) => {
      const owner = owners.find(
        ({ native }) =>
          line.startsWith(`${native};`) || line.startsWith(`${native} `),
      );
      return owner ? `${owner.prefix};${line}` : line;
    })
    .join('\n');
  return `${moved}\n${result.combinedFolded}`;
}

export function main(args = process.argv.slice(2)) {
  const { values, positionals } = parseArgs({
    args,
    allowPositionals: true,
    options: {
      'out-dir': { type: 'string' },
      'max-depth': { type: 'string', default: '32' },
      // Accepted for older commands; string payloads are never emitted.
      'include-content': { type: 'boolean', default: false },
      'clear-class-names': { type: 'boolean', default: false },
    },
  });
  if (positionals.length !== 1) {
    throw new Error(
      'Usage: node .agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs <prefix.heapsnapshot> [--out-dir <existing-dir>] [--max-depth 32] [--clear-class-names]',
    );
  }
  const source = path.resolve(positionals[0]);
  const prefix = source.replace(/\.heapsnapshot$/, '');
  if (source === prefix) throw new Error('Input must end in .heapsnapshot');
  const maxDepth = Number(values['max-depth']);
  if (!Number.isSafeInteger(maxDepth) || maxDepth < 1)
    throw new Error('--max-depth must be a positive integer');
  const name = path.basename(prefix);
  const sourceDir = path.dirname(prefix);
  // Older captures stored both inputs in one directory.
  const profileDir =
    path.basename(sourceDir) === 'v8' && !existsSync(`${prefix}.objects.json`)
      ? path.dirname(sourceDir)
      : sourceDir;
  const rustPrefix = path.join(profileDir, name);
  const identities = JSON.parse(
    readFileSync(`${rustPrefix}.objects.json`, 'utf8'),
  );
  if (identities.snapshot !== name)
    throw new Error('Rust snapshot identity does not match the V8 filename');
  const input = openSync(source, 'r');
  let snapshot;
  try {
    const buffer = Buffer.allocUnsafe(64 * 1024);
    snapshot = parseHeapSnapshot(
      () => {
        const length = readSync(input, buffer, 0, buffer.length, null);
        return length === 0 ? null : buffer.subarray(0, length);
      },
      {
        omitStringContents: true,
        preserveStrings: identities.objects.flatMap(({ address, type_name }) =>
          [address, type_name].filter((value) => typeof value === 'string'),
        ),
      },
    );
  } finally {
    closeSync(input);
  }
  const result = convert(snapshot, identities, {
    maxDepth,
    clearClassNames: values['clear-class-names'],
  });
  const rust = readFileSync(`${rustPrefix}.allocative`, 'utf8');
  const outDir = values['out-dir'] ?? path.join(profileDir, 'reports');
  if (!values['out-dir']) mkdirSync(outDir, { recursive: true });
  const output = path.join(outDir, name);
  // Refuse to overwrite captures or previous conversion results.
  const files = [
    ['.js.allocative', result.folded],
    ['.combined.allocative', combineRust(rust, result)],
    ['.links.json', `${JSON.stringify(result.report, null, 2)}\n`],
    ...Object.entries(result.categoryFolded).map(([category, folded]) => [
      `.js.${category}.allocative`,
      folded,
    ]),
  ];
  const created = [];
  try {
    for (const [suffix, data] of files) {
      const filename = `${output}${suffix}`;
      const fd = openSync(filename, 'wx');
      created.push(filename);
      try {
        writeFileSync(fd, data);
      } finally {
        closeSync(fd);
      }
    }
  } catch (error) {
    for (const filename of created) rmSync(filename, { force: true });
    throw error;
  }
  process.stdout.write(
    `${JSON.stringify({ snapshot: result.report.snapshot, represented_js_bytes: result.report.represented_bytes, excluded_v8_bytes: result.report.excluded_v8_bytes, category_bytes: result.report.category_bytes, matched_bindings: result.report.matched_bindings, unmatched_bindings: result.report.unmatched_bindings, matched_roots: result.report.matched_roots, unmatched_roots: result.report.unmatched_roots })}\n`,
  );
}

if (
  process.argv[1] &&
  pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url
) {
  main();
}
