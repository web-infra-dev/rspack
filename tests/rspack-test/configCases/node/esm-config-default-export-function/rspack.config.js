import { rspack } from '@rspack/core';

export default ({ config }, { testPath }) => ({
  target: 'node',
  mode: 'development',
  plugins: [
    new rspack.DefinePlugin({
      EXPORT_KIND: JSON.stringify('esm-default-factory'),
      FACTORY_ARGS: JSON.stringify({
        hasConfig: Boolean(config),
        hasTestPath: Boolean(testPath),
      }),
    }),
  ],
});
