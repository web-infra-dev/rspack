const { CaseSensitivePlugin } = require("@rspack/core");

module.exports = {
  plugins: [new CaseSensitivePlugin()],
};