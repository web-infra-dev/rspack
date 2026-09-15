/** @type {import("../../../../").LoaderDefinition} */
module.exports = function (source) {
  const callback = this.async();
  // The loader-provided source URL is already context-independent
  // (`rspack://` scheme); it must not be prefixed again.
  callback(null, source, {
    version: 3,
    file: 'x',
    sources: ['rspack:///./module.js'],
    sourcesContent: [source],
    names: [],
    mappings: 'AAAA',
  });
};
