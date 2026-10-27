const fs = require('node:fs');
const path = require('node:path');

const read = (name) => fs.readFileSync(path.join(__dirname, name));

it('should retain changes to a native asset Buffer', () => {
  expect(read('buffer.bin')).toEqual(Buffer.from('Abc'));
});

it('should serialize concatenated Buffers as text', () => {
  expect(read('single-buffer.txt')).toEqual(Buffer.from('body'));
  expect(read('mixed-buffer.txt')).toEqual(Buffer.from('prefix:body'));
  expect(read('buffer-child.txt')).toEqual(Buffer.from('body!'));
});

it('should retain changes to exposed source maps', () => {
  for (const method of ['map', 'sourceAndMap']) {
    expect(JSON.parse(read(`${method}-result.json`)).sources).toEqual(['changed.txt']);
    expect(JSON.parse(read(`${method}-replace-result.json`)).sources).toEqual(['replacement.txt']);
  }
});

it('should fall back for source subclasses', () => {
  expect(read('subclass-concat.txt').toString()).toBe('prefix:body');
  expect(read('subclass-replace.txt').toString()).toBe('prefix:body');
});

it('should serialize only the nested source that needs fallback', () => {
  expect(read('named-fallback.txt').toString()).toBe('outer:prefix:body:changed');
  const map = JSON.parse(read('named-map.json'));
  expect(map.sources).toEqual(['original.txt']);
  expect(map.names).toContain('replacement');
});
