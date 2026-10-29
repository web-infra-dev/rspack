import path from 'node:path';
import { SourceMapConsumer } from 'source-map';

export default async function (code, map, additionalData) {
  expect(additionalData).toEqual({ preserved: true });
  const source = path.join(this.context, 'original.less').replace(/\\/g, '/');
  expect(map.sources).toEqual([source]);
  expect(map.sourcesContent[0]).toContain('// original source');
  const consumer = await new SourceMapConsumer(map);
  const line = code.slice(0, code.indexOf('color:')).split('\n').length;
  expect(consumer.originalPositionFor({ line, column: 2 })).toMatchObject({ source, line: 4 });
  consumer.destroy?.();
  return code;
}
