const { ContainerPlugin } = require('@rspack/core').container;

module.exports = {
  entry: './index.js',
  plugins: [
    new ContainerPlugin({
      name: 'container',
      exposes: {
        './entry': {
          import: './index.js',
          layer: 'server',
        },
      },
    }),
  ],
};
