const rspackConfig = require("./rspack.config");
delete rspackConfig.builtins;

module.exports = rspackConfig;
