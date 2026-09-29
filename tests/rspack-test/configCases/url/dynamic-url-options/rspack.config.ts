import { defineConfig } from '@rspack/cli';
import { DefinePlugin, type Configuration } from '@rspack/core';

const cases: {
  module: Configuration['module'];
  enabled: boolean;
  staticEnabled?: boolean;
}[] = [
  { module: {}, enabled: true },
  { module: { parser: { javascript: { dynamicUrl: true } } }, enabled: true },
  { module: { parser: { javascript: { dynamicUrl: false } } }, enabled: false },
  {
    module: {
      parser: { javascript: { dynamicUrl: true } },
      rules: [{ test: /index\.js$/, parser: { dynamicUrl: false } }],
    },
    enabled: false,
  },
  {
    module: {
      parser: { javascript: { dynamicUrl: false } },
      rules: [{ test: /index\.js$/, parser: { dynamicUrl: true } }],
    },
    enabled: true,
  },
  {
    module: {
      parser: { 'javascript/esm': { dynamicUrl: false } },
      rules: [{ test: /index\.js$/, type: 'javascript/esm' }],
    },
    enabled: false,
  },
  {
    module: { parser: { javascript: { dynamicUrl: false, url: 'relative' } } },
    enabled: false,
  },
  {
    module: { parser: { javascript: { dynamicUrl: true, url: false } } },
    enabled: false,
    staticEnabled: false,
  },
];

export default defineConfig(
  cases.map(({ module, enabled, staticEnabled = true }) => ({
    target: 'web',
    output: { assetModuleFilename: 'bundled/[name][ext][query]' },
    module,
    plugins: [
      new DefinePlugin({
        DYNAMIC_URL_ENABLED: JSON.stringify(enabled),
        STATIC_URL_ENABLED: JSON.stringify(staticEnabled),
      }),
    ],
  })),
);
