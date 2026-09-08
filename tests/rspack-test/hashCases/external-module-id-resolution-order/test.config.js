const fs = require('fs');
const path = require('path');

/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
  validate(stats) {
    const [delayParent, delayCurrent] = stats.stats;
    const getExternalModule = (childStats) =>
      childStats
        .toJson({ all: false, modules: true, ids: true })
        .modules.find((module) => module.identifier.startsWith('external '));

    const delayParentExternal = getExternalModule(delayParent);
    const delayCurrentExternal = getExternalModule(delayCurrent);
    expect(delayParentExternal).toBeDefined();
    expect(delayCurrentExternal).toBeDefined();
    expect(delayParentExternal.identifier).toBe(delayCurrentExternal.identifier);
    expect(delayParentExternal.id).toBe(delayCurrentExternal.id);

    for (const asset of ['bundle.js', 'bundle.js.map']) {
      const delayParentSource = fs.readFileSync(
        path.resolve(__dirname, 'dist/delay-parent-request', asset),
      );
      const delayCurrentSource = fs.readFileSync(
        path.resolve(__dirname, 'dist/delay-current-request', asset),
      );
      expect(delayParentSource).toEqual(delayCurrentSource);
    }
  },
};
