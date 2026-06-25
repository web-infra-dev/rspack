const fs = require('fs');
const path = require('path');

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, 'bundle0.js'),
      'utf-8',
    );

    expect(source).toContain('__rspack_exports = __rspack_exports["default"];');
    expect(source).not.toContain(
      '__webpack_exports__ = __webpack_exports__["default"];',
    );
  },
};
