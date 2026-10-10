import fs from 'node:fs';

const cssHashes = new Map();

export default {
  // Keep distinct CSS versions visible without depending on opaque digest values.
  snapshotContent(content) {
    return content.replace(/\b[a-f0-9]{16}(?=\.css\b|"[^\n]*"\.css")/g, hash => {
      if (!cssHashes.has(hash)) cssHashes.set(hash, `CSS_HASH_${cssHashes.size}`);
      return cssHashes.get(hash);
    });
  },
  findBundle(_index, options) {
    const files = fs.readdirSync(options.output.path);
    return [
      files.find(file => /^base\..*\.css$/.test(file)),
      files.find(file => /^main\..*\.css$/.test(file)),
      'runtime.js', 'base.js', 'parent.js', 'sibling.js', 'main.js',
    ];
  },
};
