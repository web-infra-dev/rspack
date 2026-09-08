const { ModuleFederationPlugin } = require('@rspack/core').container;

// Enhanced `request` changes which module a consumer falls back to. The
// provider must discover that same module, not the config key.
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    uniqueName: 'shared-request-defaults',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'shared_request_defaults',
      shared: {
        // import omitted: fallback and provider both follow `request`
        libForRsc: {
          request: 'lib',
          shareKey: 'lib',
          version: '1.0.0',
          requiredVersion: false,
        },
        // explicit import different from request: provider follows `import`
        otherAlias: {
          request: 'other',
          import: 'other-impl',
          shareKey: 'other',
          version: '2.0.0',
          requiredVersion: false,
        },
        // import: false: consume only, never provide
        noneAlias: {
          request: 'none',
          import: false,
          shareKey: 'none',
          requiredVersion: false,
        },
      },
    }),
  ],
};
