module.exports = function(source) {
  const { certified, map, data } = this.getOptions();
  if (certified) this.experiments.portableOutput({ paths: [this.resourcePath] });
  this.callback(null, source, map ? {
    version: 3, sources: [this.resourcePath], sourcesContent: [source], names: [], mappings: 'AAAA'
  } : undefined, data ? { resource: this.resourcePath, preserved: 'producer-data' } : undefined);
};
module.exports.pitch = function() {
  if (this.getOptions().certified) this.experiments.portableOutput();
};
