module.exports = {
  module: {
    parser: {
      javascript: {
        createRequire: 'makeRequire from ./shim.js',
      },
    },
  },
};
