/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'system' },
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  externals: {
    rootExt: 'root rootExt',
    varExt: 'var varExt',
    windowExt: 'window windowExt',
  },
};
