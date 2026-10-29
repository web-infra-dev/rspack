import path from 'node:path';
import { SourceMapConsumer } from 'source-map';

export default async function (code, map) {
  expect(map).toBeTruthy();
  const filename = this.resourcePath.replace(/\\/g, '/');
  expect(map.sources).toContain(filename);
  expect(map.sources).toContain(path.join(this.context, 'theme.less').replace(/\\/g, '/'));
  expect(map.sourcesContent[map.sources.indexOf(filename)]).not.toContain('@color: red;');
  const consumer = await new SourceMapConsumer(map);
  const line = code.slice(0, code.indexOf('color:')).split('\n').length;
  const position = consumer.originalPositionFor({ line, column: 2 });
  expect(position.source).toBe(filename);
  expect(position.line).toBe(3);
  consumer.destroy?.();
  return code;
}
