import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig(
  [false, true].flatMap((pitch) =>
    [false, true].flatMap((parallel) =>
      [false, true].map((mixed) => ({
        module: {
          rules: [
            ...['string', 'buffer'].flatMap((kind) =>
              [false, true].map((raw) => ({
                resourceQuery: new RegExp(
                  `^\\?${kind}-${raw ? 'raw' : 'normal'}$`,
                ),
                use: [
                  {
                    loader: fileURLToPath(
                      new URL(
                        raw ? './raw-loader.mjs' : './normal-loader.mjs',
                        import.meta.url,
                      ),
                    ),
                    options: {},
                    parallel: parallel ? { maxWorkers: 1 } : false,
                  },
                  ...(mixed ? ['builtin:test-passthrough-loader'] : []),
                  {
                    loader: fileURLToPath(
                      new URL(
                        pitch ? './pitch-loader.mjs' : './producer-loader.mjs',
                        import.meta.url,
                      ),
                    ),
                    options: { kind },
                    parallel: parallel ? { maxWorkers: 1 } : false,
                  },
                ],
              })),
            ),
            ...[false, true].flatMap((extractSourceMap) =>
              [false, true].map((raw) => ({
                resourceQuery: new RegExp(
                  `^\\?resource-${extractSourceMap ? 'extract' : 'plain'}-${raw ? 'raw' : 'normal'}$`,
                ),
                type: 'javascript/auto',
                extractSourceMap,
                use: [
                  {
                    loader: fileURLToPath(
                      new URL(
                        raw ? './raw-loader.mjs' : './normal-loader.mjs',
                        import.meta.url,
                      ),
                    ),
                    options: {},
                    parallel: parallel ? { maxWorkers: 1 } : false,
                  },
                  ...(mixed ? ['builtin:test-passthrough-loader'] : []),
                ],
              })),
            ),
          ],
        },
      })),
    ),
  ),
);
