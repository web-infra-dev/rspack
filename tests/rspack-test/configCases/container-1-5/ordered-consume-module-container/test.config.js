const fs = require('fs');
const path = require('path');

module.exports = {
  findBundle(i) {
    return i === 0
      ? ['async-only/index.mjs']
      : ['initial/ordered.mjs', 'initial/index.mjs'];
  },
  afterExecute(configs) {
    for (const options of configs) {
      const source = fs.readFileSync(path.join(options.output.path, 'remoteEntry.mjs'), 'utf-8');
      if (options.name === 'async-only') {
        expect(source).not.toMatch(/\bawait\b/);
      } else {
        expect(source).toMatch(/\bawait\b/);
      }
    }
  },
};
