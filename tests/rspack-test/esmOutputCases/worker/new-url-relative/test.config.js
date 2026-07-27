const fs = require('fs');
const path = require('path');

module.exports = {
  afterExecute(options) {
    const mainSource = fs.readFileSync(
      path.join(options.output.path, 'main.mjs'),
      'utf-8',
    );

    expect(mainSource).toContain(
      'new URL("./worker.mjs", import.meta.url)',
    );

    const workerSource = fs.readFileSync(
      path.join(options.output.path, 'worker.mjs'),
      'utf-8',
    );

    expect(workerSource).not.toContain('registerModules');
    expect(workerSource).not.toMatch(
      /(?:__webpack_require__|__rspack_context\.r)\("\.\/worker\.js"\);/,
    );
    expect(workerSource).toContain('globalThis.__workerEntryExecuted = true;');
  },
};
