import { createRequire } from 'node:module';
import { StringDecoder } from 'node:string_decoder';

/**
 * @typedef {{
 *   node_fields: string[], node_types: (string | string[])[],
 *   edge_fields: string[], edge_types: (string | string[])[]
 * }} HeapMeta
 * @typedef {{
 *   snapshot: { meta: HeapMeta, node_count?: number, edge_count?: number },
 *   nodes: number[] | Float64Array, edges: number[] | Float64Array,
 *   strings: string[], rspack_string_contents?: { omitted_strings: number }, rspack_string_preview?: { max_length: number, truncated_strings: number },
 *   rspack_binding_nodes?: number[],
 *   rspack_v8_source_nodes?: number[], rspack_js_roots?: RspackRoot[],
 *   rspack_js_bindings?: [number, number][]
 * }} HeapSnapshot
 * @typedef {[nodeId: number, address: string, type: string]} BindingMarker
 * @typedef {'Compiler' | 'Compilation' | 'Cache'} RspackRootKind
 * @typedef {[nodeId: number, kind: RspackRootKind]} RspackRoot
 * @typedef {(typeof rspackObjectKinds)[number]} RspackObjectKind
 */

export const allocativeRootMarker = '__rspack_internal_allocativeRoot';
export const allocativeBindingMarker = '__rspack_internal_allocativeBinding';
export const allocativeObjectMarker = '__rspack_internal_allocativeObject';
const rspackObjectKinds = /** @type {const} */ ([
  'Asset',
  'Assets',
  'CompilationHooks',
  'CompilerHooks',
  'Module',
  'ModuleGraph',
  'Stats',
]);

function loadStreamJson() {
  const require = createRequire(import.meta.url);
  const { jsonParser } = require('stream-json/core/parser.js');
  const { default: Assembler } = require('stream-json/core/assembler.js');
  const chain = require('stream-chain/defs.js');
  return { jsonParser, Assembler, chain };
}

/**
 * @param {() => Uint8Array | null} readChunk
 * @param {{ maxStringLength?: number, omitStringContents?: boolean, preserveStrings?: Iterable<string> }} [options]
 * @returns {HeapSnapshot}
 */
export function parseHeapSnapshot(
  readChunk,
  {
    maxStringLength = Infinity,
    omitStringContents = false,
    preserveStrings = [],
  } = {},
) {
  if (
    maxStringLength !== Infinity &&
    (!Number.isSafeInteger(maxStringLength) || maxStringLength < 1)
  ) {
    throw new Error('maxStringLength must be a positive integer or Infinity');
  }
  // Binding identities must survive previews regardless of string-table order.
  const preserved = new Set();
  let retainedLimit = omitStringContents ? 0 : maxStringLength + 1;
  for (const value of [
    'Compiler',
    'Compilation',
    'Cache',
    ...rspackObjectKinds,
    ...preserveStrings,
  ]) {
    if (omitStringContents || value.length > maxStringLength) {
      preserved.add(value);
      retainedLimit = Math.max(retainedLimit, value.length);
    }
  }
  const { jsonParser, Assembler, chain } = loadStreamJson();
  // The tokenizer accepts the flush sentinel even though its input type only lists strings.
  const parse =
    /** @type {(input: string | typeof chain.none) => ReturnType<ReturnType<typeof jsonParser>>} */ (
      jsonParser({ streamValues: false, packStrings: false })
    );
  const decoder = new StringDecoder();
  const assembler = new Assembler();
  /** @type {{ key: 'nodes' | 'edges', values: Float64Array, index: number } | undefined} */
  let numericArray;

  /** @type {string[]} */
  const stringChunks = [];
  let readingStrings = false;
  let stringIndex = 0;
  let stringLength = 0;
  let retainedLength = 0;
  let keepFullString = true;
  let truncatedStrings = 0;
  let omittedStrings = 0;
  const protectedNames = new Set();

  // V8 writes numeric graph tables before strings. Names used by objects,
  // functions and non-indexed edges are structural, not payload previews.
  function protectNames() {
    const heap = /** @type {HeapSnapshot} */ (assembler.current);
    if (!heap.nodes || !heap.edges) {
      throw new Error('String previews require nodes and edges before strings');
    }
    const { meta } = heap.snapshot;
    const width = meta.node_fields.length;
    const typeField = meta.node_fields.indexOf('type');
    const nameField = meta.node_fields.indexOf('name');
    const types = meta.node_types[typeField];
    for (let i = 0; i < heap.nodes.length; i += width) {
      const type = types[heap.nodes[i + typeField]];
      if (
        type !== 'string' &&
        type !== 'concatenated string' &&
        type !== 'sliced string'
      ) {
        protectedNames.add(heap.nodes[i + nameField]);
      }
    }
    const edgeWidth = meta.edge_fields.length;
    const edgeTypeField = meta.edge_fields.indexOf('type');
    const edgeNameField = meta.edge_fields.indexOf('name_or_index');
    const edgeTypes = meta.edge_types[edgeTypeField];
    for (let i = 0; i < heap.edges.length; i += edgeWidth) {
      const type = edgeTypes[heap.edges[i + edgeTypeField]];
      if (type !== 'element' && type !== 'hidden') {
        protectedNames.add(heap.edges[i + edgeNameField]);
      }
    }
  }

  /** @param {string | typeof chain.none} input */
  function consume(input) {
    const batch = parse(input);
    if (batch === chain.none) return;
    for (const token of batch.values) {
      if (numericArray) {
        if (token.name === 'numberValue') {
          if (numericArray.index >= numericArray.values.length) {
            throw new Error(`Too many V8 ${numericArray.key} values`);
          }
          numericArray.values[numericArray.index++] = Number(token.value);
        } else if (token.name === 'endArray') {
          if (numericArray.index !== numericArray.values.length) {
            throw new Error(`Missing V8 ${numericArray.key} values`);
          }
          const heap = /** @type {HeapSnapshot} */ (assembler.current);
          heap[numericArray.key] = numericArray.values;
          assembler.key = null;
          numericArray = undefined;
        } else if (
          !['startNumber', 'numberChunk', 'endNumber'].includes(token.name)
        ) {
          throw new Error('Invalid V8 numeric array');
        }
        continue;
      }
      if (
        token.name === 'startArray' &&
        assembler.depth === 1 &&
        (assembler.key === 'nodes' || assembler.key === 'edges')
      ) {
        const key = assembler.key;
        const header = /** @type {HeapSnapshot} */ (assembler.current).snapshot;
        const count = key === 'nodes' ? header?.node_count : header?.edge_count;
        const fields =
          key === 'nodes' ? header?.meta.node_fields : header?.meta.edge_fields;
        if (count !== undefined && fields) {
          const length = count * fields.length;
          if (!Number.isSafeInteger(length) || length < 0) {
            throw new Error(`Invalid V8 ${key} length`);
          }
          // Ordinary growing arrays hit V8's backing-store limit on large heaps.
          // Keep exact integer values (including IDs and sizes) in bounded storage.
          numericArray = { key, values: new Float64Array(length), index: 0 };
          continue;
        }
      }
      if (
        token.name === 'startArray' &&
        assembler.depth === 1 &&
        assembler.key === 'strings'
      ) {
        if (omitStringContents || maxStringLength !== Infinity) protectNames();
        readingStrings = true;
      }
      if (token.name === 'startString') {
        keepFullString =
          !readingStrings ||
          (!omitStringContents && maxStringLength === Infinity) ||
          protectedNames.has(stringIndex);
      }
      if (token.name === 'stringChunk') {
        stringLength += token.value.length;
        const chunk = keepFullString
          ? token.value
          : token.value.slice(0, Math.max(0, retainedLimit - retainedLength));
        if (chunk) stringChunks.push(chunk);
        retainedLength += chunk.length;
      } else if (token.name === 'endString') {
        let value = stringChunks.join('');
        if (
          omitStringContents &&
          !keepFullString &&
          !(stringLength === retainedLength && preserved.has(value))
        ) {
          value = '';
          omittedStrings++;
        } else if (
          !omitStringContents &&
          !keepFullString &&
          stringLength > maxStringLength &&
          !(stringLength === retainedLength && preserved.has(value))
        ) {
          let end = maxStringLength;
          const last = value.charCodeAt(end - 1);
          const next = value.charCodeAt(end);
          if (
            last >= 0xd800 &&
            last <= 0xdbff &&
            next >= 0xdc00 &&
            next <= 0xdfff
          )
            end--;
          value = value.slice(0, end) + '…';
          truncatedStrings++;
        }
        assembler.consume({ name: 'stringValue', value });
        stringChunks.length = 0;
        stringLength = 0;
        retainedLength = 0;
        if (readingStrings) stringIndex++;
      } else {
        assembler.consume(token);
        if (readingStrings && token.name === 'endArray') readingStrings = false;
      }
    }
  }

  let chunk;
  while ((chunk = readChunk()) !== null) {
    consume(decoder.write(chunk));
  }
  consume(decoder.end());
  consume(chain.none);
  if (numericArray || !assembler.done || !assembler.current) {
    throw new Error('Invalid V8 heap snapshot');
  }
  const heap = /** @type {HeapSnapshot} */ (assembler.current);
  if (omitStringContents) {
    heap.rspack_string_contents = { omitted_strings: omittedStrings };
  } else if (maxStringLength !== Infinity) {
    heap.rspack_string_preview = {
      max_length: maxStringLength,
      truncated_strings: truncatedStrings,
    };
  }
  return heap;
}

/** @param {string} kind @returns {kind is RspackRootKind} */
export function isRspackRootKind(kind) {
  return kind === 'Compiler' || kind === 'Compilation' || kind === 'Cache';
}

/** @param {HeapSnapshot} snapshot */
export function readHeapGraph(snapshot) {
  const { meta } = snapshot.snapshot;
  const { nodes, edges, strings } = snapshot;
  const nodeWidth = meta.node_fields.length;
  const edgeWidth = meta.edge_fields.length;
  /** @param {string[]} fields @param {string} name */
  const field = (fields, name) => {
    const index = fields.indexOf(name);
    if (index < 0) throw new Error(`Missing V8 snapshot field: ${name}`);
    return index;
  };
  const nt = field(meta.node_fields, 'type');
  const nn = field(meta.node_fields, 'name');
  const ni = field(meta.node_fields, 'id');
  const ns = field(meta.node_fields, 'self_size');
  const nc = field(meta.node_fields, 'edge_count');
  const et = field(meta.edge_fields, 'type');
  const en = field(meta.edge_fields, 'name_or_index');
  const to = field(meta.edge_fields, 'to_node');
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

  return {
    count,
    edgeWidth,
    offsets,
    nodeName: (/** @type {number} */ i) => strings[nodes[i * nodeWidth + nn]],
    nodeType: (/** @type {number} */ i) => nodeTypes[nodes[i * nodeWidth + nt]],
    nodeId: (/** @type {number} */ i) => nodes[i * nodeWidth + ni],
    nodeSize: (/** @type {number} */ i) => nodes[i * nodeWidth + ns],
    edgeType: (/** @type {number} */ e) => edgeTypes[edges[e + et]],
    edgeName: (/** @type {number} */ e) => strings[edges[e + en]],
    target(/** @type {number} */ e) {
      const offset = edges[e + to];
      if (offset % nodeWidth || offset < 0 || offset >= nodes.length) {
        throw new Error(`Invalid V8 edge target: ${offset}`);
      }
      return offset / nodeWidth;
    },
  };
}

/** @param {ReturnType<typeof readHeapGraph>} graph @param {number} node */
export function readBindingMarker(graph, node) {
  let pointer;
  let type;
  for (
    let e = graph.offsets[node];
    e < graph.offsets[node + 1];
    e += graph.edgeWidth
  ) {
    if (graph.edgeType(e) !== 'property') continue;
    const name = graph.edgeName(e);
    if (name !== '__rspackRustPtr' && name !== '__rspackRustType') continue;
    const value = graph.target(e);
    if (graph.nodeType(value) !== 'string') continue;
    if (name === '__rspackRustPtr') pointer = graph.nodeName(value);
    else type = graph.nodeName(value);
  }
  if (!pointer || !type || !/^0x[0-9a-f]+$/i.test(pointer)) return undefined;
  /** @type {BindingMarker} */
  const marker = [graph.nodeId(node), pointer.toLowerCase(), type];
  return marker;
}

/** @param {string} type */
function isString(type) {
  return (
    type === 'string' ||
    type === 'concatenated string' ||
    type === 'sliced string'
  );
}

/** @param {ReturnType<typeof readHeapGraph>} graph */
export function readHeapCategories(graph) {
  const sourceNodes = new Set();
  /** @type {RspackRoot[]} */
  const rspackRoots = [];
  /** @type {[number, number][]} */
  const rspackBindings = [];
  /** @type {Map<number, RspackObjectKind>} */
  const rspackObjects = new Map();
  for (let i = 0; i < graph.count; i++) {
    const type = graph.nodeType(i);
    if (type !== 'code' && type !== 'object') continue;
    for (
      let edge = graph.offsets[i];
      edge < graph.offsets[i + 1];
      edge += graph.edgeWidth
    ) {
      const name = graph.edgeName(edge);
      const edgeType = graph.edgeType(edge);
      // V8 exposes Script::source through a named internal edge.
      if (type === 'code' && edgeType === 'internal' && name === 'source') {
        const target = graph.target(edge);
        if (isString(graph.nodeType(target))) sourceNodes.add(target);
      } else if (
        type === 'object' &&
        edgeType === 'property' &&
        (name === allocativeRootMarker || name === allocativeObjectMarker)
      ) {
        const target = graph.target(edge);
        if (graph.nodeType(target) !== 'string') continue;
        const kind = graph.nodeName(target);
        if (name === allocativeRootMarker) {
          if (isRspackRootKind(kind)) rspackRoots.push([graph.nodeId(i), kind]);
        } else {
          const objectKind = rspackObjectKinds.find((value) => value === kind);
          if (objectKind) rspackObjects.set(graph.nodeId(i), objectKind);
        }
      } else if (
        type === 'object' &&
        edgeType === 'property' &&
        name === allocativeBindingMarker
      ) {
        const target = graph.target(edge);
        if (
          graph.nodeType(target) === 'object' &&
          readBindingMarker(graph, target)
        ) {
          rspackBindings.push([graph.nodeId(i), graph.nodeId(target)]);
        }
      }
    }
  }

  // Follow only V8's string representation edges. A source string may store
  // its bytes in other strings or an external backing store.
  for (const node of sourceNodes) {
    if (!isString(graph.nodeType(node))) continue;
    for (
      let edge = graph.offsets[node];
      edge < graph.offsets[node + 1];
      edge += graph.edgeWidth
    ) {
      if (graph.edgeType(edge) !== 'internal') continue;
      const name = graph.edgeName(edge);
      const target = graph.target(edge);
      const type = graph.nodeType(target);
      if (
        (isString(type) &&
          (name === 'first' ||
            name === 'second' ||
            name === 'parent' ||
            name === 'actual')) ||
        (type === 'native' && name?.startsWith('backing_store'))
      ) {
        sourceNodes.add(target);
      }
    }
  }
  return {
    sourceNodeIds: Array.from(sourceNodes, graph.nodeId),
    rspackRoots,
    rspackBindings,
    rspackObjects,
  };
}
