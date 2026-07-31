const path = require('path');

const calls = [];
const entryApi = { source: 'entry' };

globalThis['@rstest/core/import-meta'] = (filename) => {
  calls.push(filename);
  return path.basename(filename) === 'index.js' ? entryApi : undefined;
};

const direct = import.meta.rstest;
const optional = import.meta.rstest?.source;
const property = import.meta.rstest.source;
const type = typeof import.meta.rstest;
let branch = false;
if (import.meta.rstest) {
  branch = true;
}

module.exports = {
  branch,
  calls,
  direct,
  imported: require('./imported'),
  optional,
  property,
  type,
};
