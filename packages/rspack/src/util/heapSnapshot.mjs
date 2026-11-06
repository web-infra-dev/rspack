/**
 * @typedef {{
 *   node_fields: string[], node_types: (string | string[])[],
 *   edge_fields: string[], edge_types: (string | string[])[]
 * }} HeapMeta
 * @typedef {{
 *   snapshot: { meta: HeapMeta }, nodes: number[], edges: number[],
 *   strings: string[], rspack_binding_nodes?: number[],
 *   rspack_v8_source_nodes?: number[], rspack_js_roots?: RspackRoot[]
 * }} HeapSnapshot
 * @typedef {[nodeId: number, address: string, type: string]} BindingMarker
 * @typedef {'Compiler' | 'Compilation' | 'Cache'} RspackRootKind
 * @typedef {[nodeId: number, kind: RspackRootKind]} RspackRoot
 */

export const allocativeRootMarker = '__rspack_internal_allocativeRoot';

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
    nodeWidth,
    edgeWidth,
    offsets,
    nodeColumns: [nt, ni, ns, nc],
    edgeColumns: [et, to],
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
  return { sourceNodeIds: Array.from(sourceNodes, graph.nodeId), rspackRoots };
}

/** @param {number[]} values @param {number} width @param {number[]} columns */
function redactColumns(values, width, columns) {
  for (let col = 0; col < width; col++) {
    if (columns.includes(col)) continue;
    for (let i = col; i < values.length; i += width) values[i] = 0;
  }
}

/** @param {HeapSnapshot} heap */
export function redactHeapSnapshot(heap) {
  const graph = readHeapGraph(heap);
  const { sourceNodeIds, rspackRoots } = readHeapCategories(graph);
  const markers = [];
  for (let i = 0; i < graph.count; i++) {
    const marker = readBindingMarker(graph, i);
    if (marker) markers.push(marker);
  }

  // Keep only the V8 graph schema, sizes, and reference topology. The marker
  // values are returned to Rust for in-memory matching and are never serialized.
  redactColumns(heap.nodes, graph.nodeWidth, graph.nodeColumns);
  redactColumns(heap.edges, graph.edgeWidth, graph.edgeColumns);
  const redacted = {
    snapshot: {
      meta: heap.snapshot.meta,
      node_count: graph.count,
      edge_count: heap.edges.length / graph.edgeWidth,
      trace_function_count: 0,
    },
    nodes: heap.nodes,
    edges: heap.edges,
    trace_function_infos: [],
    trace_tree: [],
    samples: [],
    locations: [],
    strings: [''],
    rspack_binding_nodes: markers.map(([id]) => id),
    rspack_v8_source_nodes: sourceNodeIds,
    rspack_js_roots: rspackRoots,
  };
  return { redacted, markers };
}
