const path = require('path');

const COMPILATION_LOG = 'compilation logger warning';
const explicitLogging = process.env.EXPLICIT_LOGGING === 'true';
const partialStats = process.env.PARTIAL_STATS === 'true';
const statsPreset = process.env.STATS_PRESET;
const allStats = process.env.ALL_STATS === 'true';

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist/logging'),
  },
  ...(explicitLogging
    ? {
        stats: {
          all: false,
          logging: 'warn',
        },
      }
    : statsPreset
      ? {
          stats:
            statsPreset === 'object'
              ? {
                  preset: 'verbose',
                }
              : 'verbose',
        }
      : allStats
        ? {
            stats: {
              all: true,
            },
          }
        : partialStats
          ? {
              stats: {
                timings: false,
              },
            }
          : {}),
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap(
          'LoggingDefaultsPlugin',
          (compilation) => {
            compilation
              .getLogger('LoggingDefaultsPlugin')
              .warn(COMPILATION_LOG);
          },
        );
      },
    },
  ],
};
