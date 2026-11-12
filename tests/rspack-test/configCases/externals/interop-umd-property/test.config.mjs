export default {
  modules: {
    dep: {
      __esModule: true,
      default: 'the default',
      named: 42,
      nested: { __esModule: true, default: 'nested default', named: 43 },
    },
  },
};
