const fs = require('fs');
const path = require('path');

const assetName = value => (Array.isArray(value) ? value[0] : value);

/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
  validate(stats) {
    const before = stats.stats[0].toJson({ assets: true });
    const after = stats.stats[1].toJson({ assets: true });

    const mainBefore = assetName(before.assetsByChunkName.main);
    const mainAfter = assetName(after.assetsByChunkName.main);
    const runtimeBefore = assetName(before.assetsByChunkName.runtime);
    const runtimeAfter = assetName(after.assetsByChunkName.runtime);

    expect(before.hash).not.toBe(after.hash);
    expect(mainBefore).not.toBe(mainAfter);
    expect(runtimeBefore).not.toBe(runtimeAfter);

    const runtimeSourceBefore = fs.readFileSync(
      path.resolve(__dirname, 'dist/version0', runtimeBefore),
      'utf-8',
    );
    const runtimeSourceAfter = fs.readFileSync(
      path.resolve(__dirname, 'dist/version1', runtimeAfter),
      'utf-8',
    );

    const mainHashBefore = mainBefore.slice('main.'.length, -'.js'.length);
    const mainHashAfter = mainAfter.slice('main.'.length, -'.js'.length);
    expect(runtimeSourceBefore).toContain(`.${mainHashBefore}.js`);
    expect(runtimeSourceAfter).toContain(`.${mainHashAfter}.js`);
    expect(runtimeSourceBefore).not.toContain('webpack/runtime/get_full_hash');
    expect(runtimeSourceAfter).not.toContain('webpack/runtime/get_full_hash');
  },
};
