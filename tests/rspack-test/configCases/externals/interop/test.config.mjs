export default {
  modules: {
    dep: {
      __esModule: true,
      default: 'the default',
      namedExport: 42,
      nested: { __esModule: true, default: 'nested default', namedExport: 43 },
    },
    unflagged: { default: 'unflagged default', namedExport: 44 },
  },
};
