import { readFileSync } from 'fs';
import type { RawSourceMap } from 'source-map';
import { SourceMapConsumer } from 'source-map';
import remapping from '../../../src/remapping';

function read(filename: string): string {
  return readFileSync(`${__dirname}/files/${filename}`, 'utf8');
}

describe('transpile then concatenate', () => {
  test('concated sections point to source files', () => {
    const map = read('bundle.js.map');
    const remapped = remapping(map, (file) => {
      return file.endsWith('.mjs') ? null : read(`${file}.map`);
    });

    const consumer = new SourceMapConsumer(remapped as unknown as RawSourceMap);

    const a = consumer.originalPositionFor({
      column: 11,
      line: 21,
    });
    expect(a).toMatchObject({
      column: 21,
      line: 17,
      source: 'a.mjs',
    });

    const b = consumer.originalPositionFor({
      column: 11,
      line: 40,
    });
    expect(b).toMatchObject({
      column: 21,
      line: 17,
      source: 'b.mjs',
    });
  });

  test('inherits sourcesContent of original sources', () => {
    const map = read('bundle.js.map');
    const remapped = remapping(map, (file) => {
      return file.endsWith('.mjs') ? null : read(`${file}.map`);
    });

    expect(remapped.sourcesContent).toEqual([read('a.mjs'), read('b.mjs')]);
  });
});
