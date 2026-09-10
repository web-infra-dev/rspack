import { define } from 'rstack';
import { defineInlineProject } from 'rstack/test';
import { e2eConfig } from './config.ts';

const { pool, reporters, ...projectConfig } = e2eConfig(60_000, 4);

define.test({
  pool,
  reporters,
  projects: [false, true].map((incremental) =>
    defineInlineProject({
      ...projectConfig,
      name: incremental ? 'chromium-incremental' : 'chromium',
      env: {
        // Rstest defaults to "test", but react-refresh/babel requires development.
        NODE_ENV: 'development',
        RSPACK_E2E_INCREMENTAL: String(incremental),
      },
      output: {
        // Native ESM configs and bundled tests must share one native core instance.
        externals: { '@rspack/core': 'commonjs @rspack/core' },
      },
    }),
  ),
});
