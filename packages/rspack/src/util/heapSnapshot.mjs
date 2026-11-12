/* global SUPPORTS_ALLOCATIVE: readonly */

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
 *   strings: string[], rspack_binding_nodes?: number[],
 *   rspack_v8_source_nodes?: number[], rspack_js_roots?: RspackRoot[],
 *   rspack_js_bindings?: [number, number][]
 * }} HeapSnapshot
 * @typedef {[nodeId: number, address: string, type: string]} BindingMarker
 * @typedef {'Compiler' | 'Compilation' | 'Cache'} RspackRootKind
 * @typedef {[nodeId: number, kind: RspackRootKind]} RspackRoot
 */

export const allocativeRootMarker = '__rspack_internal_allocativeRoot';
export const allocativeBindingMarker = '__rspack_internal_allocativeBinding';

function loadStreamJson() {
  // The standalone converter runs this module directly, without Rslib's define.
  if (typeof SUPPORTS_ALLOCATIVE === 'undefined' || SUPPORTS_ALLOCATIVE) {
    const require = createRequire(import.meta.url);
    const { jsonParser } =
      /** @type {typeof import('stream-json/core/parser.js')} */ (
        require('stream-json/core/parser.js')
      );
    const { default: Assembler } =
      /** @type {typeof import('stream-json/core/assembler.js')} */ (
        require('stream-json/core/assembler.js')
      );
    const chain = /** @type {typeof import('stream-chain/defs.js')} */ (
      require('stream-chain/defs.js')
    );
    return { jsonParser, Assembler, chain };
  }
  throw new Error('Allocative heap snapshots are disabled in this build');
}

/**
 * @param {() => Uint8Array | null} readChunk
 * @returns {HeapSnapshot}
 */
export function parseHeapSnapshot(readChunk) {
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
      if (token.name === 'stringChunk') {
        stringChunks.push(token.value);
      } else if (token.name === 'endString') {
        assembler.consume({
          name: 'stringValue',
          value: stringChunks.join(''),
        });
        stringChunks.length = 0;
      } else {
        assembler.consume(token);
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
  return /** @type {HeapSnapshot} */ (assembler.current);
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
        name === allocativeRootMarker
      ) {
        const target = graph.target(edge);
        if (graph.nodeType(target) !== 'string') continue;
        const kind = graph.nodeName(target);
        if (isRspackRootKind(kind)) {
          rspackRoots.push([graph.nodeId(i), kind]);
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
  };
}
