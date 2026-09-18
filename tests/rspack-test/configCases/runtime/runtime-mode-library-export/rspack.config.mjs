export default {
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
