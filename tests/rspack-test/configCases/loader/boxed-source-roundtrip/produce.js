function produce() {
  this.cacheable(false);
  this.addDependency(__filename);
  this.__internal__setParseMeta('boxed-source', 'producer');
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
  if (this.resourceQuery === '?pitch') return produce.call(this);
};
