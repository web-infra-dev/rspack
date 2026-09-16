export default {
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  optimization: {
    usedExports: true,
    concatenateModules: true,
  },
};
