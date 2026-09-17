export default {
  externals: {
    fs: 'module fs',
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
  },
};
