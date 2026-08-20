/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    json: 'var JSON',
    promise: 'var Promise',
    url: 'var URL',
    'url-search-params': 'var URLSearchParams',
    symbol: 'var Symbol',
    reflect: 'var Reflect',
    'global-this': 'var globalThis',
  },
};
