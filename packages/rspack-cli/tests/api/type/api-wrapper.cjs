const rspackCjsDefaultRequire = require('@rspack/core');
const {
  rspack: rspackCjsNamedRequire,
  webpack: webpackCjsNamedRequire,
} = require('@rspack/core');

module.exports = {
  rspackCjsDefaultRequire,
  rspackCjsNamedRequire,
  webpackCjsNamedRequire,
};
