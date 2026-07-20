const fs = require('fs');
const path = require('path');

module.exports = {
  snapshotFileFilter(file) {
    return file === 'worker.mjs';
  },
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, 'worker.mjs'),
      'utf-8',
    );

    expect(source).toMatch(
      /(?:__webpack_require__|__rspack_context\.r)\("\.\/worker\.js"\);/,
    );
  },
};
