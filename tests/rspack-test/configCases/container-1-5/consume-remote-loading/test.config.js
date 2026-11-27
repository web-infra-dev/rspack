const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const path = require('node:path');

module.exports = {
  findBundle(_index, options) {
    return path.join(options.output.path, 'main.js');
  },
  afterExecute(configs) {
    for (const { output } of configs) {
      const value = execFileSync(process.execPath, [
        '-e',
        'globalThis.it = () => {}; console.log(require(process.argv[1]));',
        path.join(output.path, 'main.js'),
      ], { encoding: 'utf8' });
      assert.equal(value.trim(), 'local');
    }
  },
};
