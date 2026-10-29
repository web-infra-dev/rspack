const unicodePrefix = '😀';
const sourceValue = '__REPLACE_ME__';

it('should preserve nested structured source updates', () => {
  expect(unicodePrefix).toBe('X');
  expect(sourceValue).toBe('updated');
  expect(globalThis.__STRUCTURED_SOURCE_PREFIX__).toBe(true);
});
