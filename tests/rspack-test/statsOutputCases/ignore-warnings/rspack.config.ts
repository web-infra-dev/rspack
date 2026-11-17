import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';
import { config, WebpackError, type Compilation } from '@rspack/core';

const compilation = {} as Compilation;

for (const flags of ['', 'g', 'y', 'gy']) {
  for (const field of ['shorthand', 'message', 'module', 'file']) {
    const pattern = new RegExp('warning', flags);
    pattern.lastIndex = 3;
    const rule = field === 'shorthand' ? pattern : { [field]: pattern };
    const [filter] = config.getNormalizedRspackOptions({
      ignoreWarnings: [rule],
    }).ignoreWarnings!;
    const warning = Object.assign(new WebpackError('warning'), {
      file: 'warning',
      module: { readableIdentifier: () => 'warning' },
    });
    for (let i = 0; i < 3; i++) {
      assert.equal(filter(warning, compilation), true);
    }
    assert.equal(pattern.lastIndex, 3);
    assert.equal(filter(new WebpackError('different'), compilation), false);
  }
}

export default defineConfig({
  entry: './index.js',
  ignoreWarnings: [
    {
      module: /module2\.js\?[34]/g,
    },
    {
      module: /[13]/g,
      message: /homepage/g,
    },
    /The 'mode' option has not been set/,
    (warning) => {
      return warning.module?.identifier().endsWith('?2') ?? false;
    },
  ],
  stats: {
    assets: true,
    modules: true,
  },
});
