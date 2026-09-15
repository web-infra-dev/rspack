module.exports = {
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    library: {
      name: 'RuntimeModeLibraryExport',
      type: 'umd',
      export: 'default',
    },
  },
};
