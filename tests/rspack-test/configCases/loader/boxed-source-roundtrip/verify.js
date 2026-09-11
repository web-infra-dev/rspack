module.exports = function (content, sourceMap, additionalData) {
  expect(Buffer.isBuffer(content)).toBe(true);
  expect(content.equals(Buffer.from([0, 255, 254, 128, 10]))).toBe(true);
  expect(sourceMap.sources).toEqual(['original.js']);
  expect(sourceMap.mappings).toBe('AAAA');
  expect(this.getDependencies()).toContain(require.resolve('./produce.js'));
  this.__internal__setParseMeta('boxed-source', 'verified');
  return `module.exports = ${JSON.stringify({ hex: content.toString('hex'), value: additionalData.value() })}`;
};
module.exports.raw = true;
