import { readFileSync } from 'fs';
import type { RawSourceMap } from 'source-map';
import { SourceMapConsumer } from 'source-map';
import remapping from '../../../src/remapping';

function read(filename: string): string {
  return readFileSync(`${__dirname}/files/${filename}`, 'utf8');
}

describe('transpile then minify', () => {
  test('minify a transpiled source map', () => {
    const map = read('helloworld.min.js.map');
    const remapped = remapping(map, (file) => {
      return file.endsWith('.mjs') ? null : read(`${file}.map`);
    });

    const consumer = new SourceMapConsumer(remapped as unknown as RawSourceMap);
    const alert = consumer.originalPositionFor({
      column: 47,
      line: 16,
    });
    expect(alert).toEqual({
      column: 20,
      line: 19,
      name: 'alert',
      source: 'helloworld.mjs',
    });
  });

  test('inherits sourcesContent of original source', () => {
    const map = read('helloworld.min.js.map');
    const remapped = remapping(map, (file) => {
      return file.endsWith('.mjs') ? null : read(`${file}.map`);
    });

    expect(remapped.sourcesContent).toEqual([read('helloworld.mjs')]);
  });
});
