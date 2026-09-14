module.exports = function (content, sourceMap, additionalData) {
  expect(this.data.owner).toBe('verify');
  expect(this.data.fromPitch).toBeUndefined();
  expect(Buffer.isBuffer(content)).toBe(true);
  expect(content.equals(Buffer.from([0, 255, 254, 128, 10]))).toBe(true);
  expect(sourceMap.sources).toEqual(['original.js']);
  expect(sourceMap.mappings).toBe('AAAA');
  expect(this.getDependencies()).toContain(require.resolve('./produce.js'));
  this.__internal__setParseMeta('loader-state', 'verified');
  return `module.exports = ${JSON.stringify({ hex: content.toString('hex'), value: additionalData.value() })}`;
};
module.exports.raw = true;

module.exports.pitch = function () {
  this.data = { owner: 'verify' };
};
