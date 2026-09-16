const path = require('path');

/** @type {import("../../../../").LoaderDefinition} */
module.exports = function (source) {
  const callback = this.async();
  const name = path.basename(this.resourcePath);
  // Loader-provided source URLs that are already context-independent
  // (`rspack://` scheme or standard URLs) must not be prefixed again, and
  // absolute sources must override `sourceRoot` instead of being joined
  // onto it.
  const sourceUrl = name.startsWith('https')
    ? 'https://cdn.example/https.js'
    : name.startsWith('data')
      ? 'data:text/javascript,console.log(1)'
      : 'rspack:///./module.js';
  callback(null, source, {
    version: 3,
    file: 'x',
    sources: [sourceUrl],
    sourceRoot: name.startsWith('module') ? '/src' : undefined,
    sourcesContent: [source],
    names: [],
    mappings: 'AAAA',
  });
};
