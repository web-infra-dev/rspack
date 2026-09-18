import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
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
