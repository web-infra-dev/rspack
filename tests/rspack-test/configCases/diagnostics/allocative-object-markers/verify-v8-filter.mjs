import assert from 'node:assert/strict';
import { convert, combineRust } from '../../../../../.agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs';

export function verifyV8Filter() {
  const types = ['synthetic', 'object', 'code', 'hidden', 'object shape', 'array', 'native', 'string', 'closure'];
  const entries = [
    ['synthetic', '(GC roots)', 1],
    ['code', 'Script', 2],
    ['hidden', 'system / FunctionTemplateInfo', 4],
    ['object shape', 'system / Map', 8],
    ['object', 'BusinessObject', 16],
    ['array', '(object elements)', 32],
    ['array', '', 64],
    ['native', 'system / JSArrayBufferData', 128],
    ['string', 'private source text', 256],
    ['object', 'Map', 512],
    ['object', 'Set', 1024],
    ['object', 'Buffer', 2048],
    ['hidden', 'system / PropertyArray', 4096],
    ['closure', 'businessCallback', 8192],
  ];
  // Every business node is reached through excluded engine metadata. Storage
  // shared by Map/Set and the cycle must still contribute its bytes only once.
  const edges = [
    [[1, 'child']], [[2, 'child'], [8, 'source']], [[3, 'child']], [[4, 'child']],
    [[5, 'elements'], [9, 'map'], [10, 'set'], [11, 'buffer'], [12, 'properties'], [13, 'callback']],
    [], [], [], [], [[6, 'table']], [[6, 'table']], [[7, 'backing_store']], [[4, 'cycle']], [],
  ];
  const strings = [];
  const string = value => { let i = strings.indexOf(value); if (i < 0) { i = strings.length; strings.push(value); } return i; };
  const snapshot = {
    snapshot: { meta: {
      node_fields: ['type', 'name', 'id', 'self_size', 'edge_count'],
      node_types: [types, 'string', 'number', 'number', 'number'],
      edge_fields: ['type', 'name_or_index', 'to_node'],
      edge_types: [['internal'], 'string_or_number', 'node'],
    } }, nodes: [], edges: [], strings,
  };
  for (const [i, [type, name, size]] of entries.entries()) {
    snapshot.nodes.push(types.indexOf(type), string(name), i * 2 + 1, size, edges[i].length);
    for (const [target, name] of edges[i]) snapshot.edges.push(0, string(name), target * 5);
  }
  const result = convert(snapshot, { objects: [] });
  const total = entries.reduce((sum, entry) => sum + entry[2], 0);
  const excluded = 1 + 2 + 4 + 8 + 256;
  const weight = text => text.trim().split('\n').filter(Boolean).reduce((sum, line) => sum + Number(line.slice(line.lastIndexOf(' ') + 1)), 0);
  assert.equal(result.report.snapshot_bytes, total);
  assert.equal(result.report.excluded_v8_bytes, excluded);
  assert.equal(result.report.represented_bytes, total - excluded);
  assert.equal(weight(result.folded), total - excluded);
  assert.equal(weight(combineRust('Rust;owner 42\n', result)), total - excluded + 42);
  assert.equal(Object.values(result.report.category_bytes).reduce((a, b) => a + b, 0), total - excluded);
  assert.doesNotMatch(result.folded, /V8 source and code|GC roots|code: Script|FunctionTemplateInfo|system \/ Map|private source text|hidden:|object shape:/);
  for (const name of ['BusinessObject', 'Map', 'Set', 'Buffer', 'businessCallback']) assert(result.folded.includes(name));
  assert(!Object.hasOwn(result.categoryFolded, 'v8_code'));
  // Filtered ancestors do not consume the visible path depth budget.
  assert(convert(snapshot, { objects: [] }, { maxDepth: 2 }).folded.includes('BusinessObject'));
}
