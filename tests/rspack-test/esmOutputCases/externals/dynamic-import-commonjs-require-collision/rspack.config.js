module.exports = {
  output: {
    importFunctionName: 'import.meta.__customImport__',
  },
  externals: {
    os: 'commonjs os',
  },
};
