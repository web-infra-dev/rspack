const fs = require('fs');
const path = require('path');

module.exports = {
  snapshotFileFilter(file) {
    return file === 'worker.mjs' || file === 'worker-cjs.mjs';
  },
  afterExecute(options) {
    const esmSource = fs.readFileSync(
      path.join(options.output.path, 'worker.mjs'),
      'utf-8',
    );

    expect(esmSource).not.toContain('registerModules');
    expect(esmSource).not.toMatch(
      /(?:__webpack_require__|__rspack_context\.r)\("\.\/worker\.js"\);/,
    );
    expect(esmSource).toContain('globalThis.__workerEntryExecuted = true;');

    const commonJsSource = fs.readFileSync(
      path.join(options.output.path, 'worker-cjs.mjs'),
      'utf-8',
    );

    expect(commonJsSource).toMatch(
      /(?:__webpack_require__|__rspack_context\.r)\("\.\/worker-cjs\.js"\);/,
    );
    expect(commonJsSource).toContain(
      'globalThis.__workerCommonJsEntryExecuted = true;',
    );
  },
};
