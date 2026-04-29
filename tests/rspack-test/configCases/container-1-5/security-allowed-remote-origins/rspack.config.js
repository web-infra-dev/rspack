const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: '[name].js',
    uniqueName: 'security-allowed-remote-origins',
  },
  plugins: [
    new ModuleFederationPlugin({
      security: {
        allowedRemoteOrigins: ['localhost', 'https://cdn.example.com'],
      },
      shared: {
        react: {
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
          version: '0.1.2',
        },
      },
    }),
  ],
};
