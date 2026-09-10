const { rspack } = require('@rspack/core');

const configFactory = ({ config }, { testPath }) => ({
  target: 'node',
  mode: 'development',
  plugins: [
    new rspack.DefinePlugin({
      EXPORT_KIND: JSON.stringify('commonjs-factory'),
      FACTORY_ARGS: JSON.stringify({
        hasConfig: Boolean(config),
        hasTestPath: Boolean(testPath),
      }),
    }),
  ],
});

configFactory.default = {
  plugins: [
    new rspack.DefinePlugin({
      EXPORT_KIND: JSON.stringify('wrong-default'),
    }),
  ],
};

module.exports = configFactory;
