function produce() {
  expect(this.data.fromPitch).toBe(true);
  this.cacheable(false);
  this.addDependency(__filename);
  this.__internal__setParseMeta('loader-state', 'producer');
  this.callback(null, Buffer.from([0, 255, 254, 128, 10]), {
    version: 3,
    names: [],
    sources: ['original.js'],
    sourcesContent: ['original'],
    mappings: 'AAAA',
  }, { value: () => 42 });
}

module.exports = produce;
module.exports.pitch = function () {
  this.data = { fromPitch: true };
  if (this.resourceQuery === '?pitch') return produce.call(this);
};
