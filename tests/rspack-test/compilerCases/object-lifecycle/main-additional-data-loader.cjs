let collected = false;
const registry = new FinalizationRegistry(() => {
  collected = true;
});

module.exports = function mainAdditionalDataLoader(source, sourceMap, additionalData) {
  registry.register(additionalData, undefined);
  this.callback(null, source, sourceMap);
};

module.exports.wasCollected = () => collected;
