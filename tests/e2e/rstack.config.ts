import { define } from 'rstack';
import { e2eConfig } from './config.ts';

define.test({
  ...e2eConfig(60_000, 4),
  name: 'chromium',
  env: {
    // Rstest defaults to "test", but react-refresh/babel requires development.
    NODE_ENV: 'development',
  },
  output: {
    // Native ESM configs and bundled tests must share one native core instance.
    externals: { '@rspack/core': 'commonjs @rspack/core' },
  },
});
