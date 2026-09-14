module.exports = [false, true].flatMap((pitch) =>
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
                  loader: require.resolve(
                    raw ? './raw-loader' : './normal-loader',
                  ),
                  options: {},
                  parallel: parallel ? { maxWorkers: 1 } : false,
                },
                ...(mixed ? ['builtin:test-passthrough-loader'] : []),
                {
                  loader: require.resolve(
                    pitch ? './pitch-loader' : './producer-loader',
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
                  loader: require.resolve(
                    raw ? './raw-loader' : './normal-loader',
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
);
