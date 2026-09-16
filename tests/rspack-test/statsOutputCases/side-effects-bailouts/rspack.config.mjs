/** @type {import('@rspack/core').Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  mode: 'production',
  stats: {
    assets: true,
    chunks: true,
    children: true,
    modules: true,
    optimizationBailout: true,
    reasons: true,
    ids: true,
    providedExports: true,
    usedExports: true,
  },
};
