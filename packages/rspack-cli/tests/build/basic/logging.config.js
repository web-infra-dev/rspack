const path = require('path');

const COMPILATION_LOG = 'compilation logger warning';
const INFRASTRUCTURE_LOG = 'infrastructure logger warning';
const explicitLogging = process.env.EXPLICIT_LOGGING === 'true';

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
