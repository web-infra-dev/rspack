/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
  findBundle: function (_i, options) {
    const uniqueName = (options.output && options.output.uniqueName) || '';
    if (uniqueName.includes('rsc-mf-shared-use-client-client')) {
      return;
    }
    return './main.js';
  },
};
