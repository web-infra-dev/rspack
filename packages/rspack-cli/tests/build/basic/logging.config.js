const path = require('path');

const COMPILATION_LOG = 'compilation logger warning';
const INFRASTRUCTURE_LOG = 'infrastructure logger warning';
const explicitLogging = process.env.EXPLICIT_LOGGING === 'true';
const partialStats = process.env.PARTIAL_STATS === 'true';
const statsPreset = process.env.STATS_PRESET;

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist/logging'),
  },
  ...(explicitLogging
    ? {
        infrastructureLogging: {
          level: 'warn',
        },
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
        compiler
          .getInfrastructureLogger('LoggingDefaultsPlugin')
          .warn(INFRASTRUCTURE_LOG);
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
