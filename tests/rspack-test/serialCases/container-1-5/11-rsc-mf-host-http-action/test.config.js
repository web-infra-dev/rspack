/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
  findBundle: function (_i, options) {
    const uniqueName = (options.output && options.output.uniqueName) || '';
    if (uniqueName.includes('rsc-mf-action-host-server')) {
      return './host/main.js';
    }
  },
};
