import path from 'node:path';
export default function(code, map) {
  const file = path.join(this.context, 'node_modules/exported/entry.less').replace(/\\/g, '/');
  expect(map.sources).toContain(file);
  expect(map.sourcesContent[map.sources.indexOf(file)]).toContain('.paint()');
  return code;
}
