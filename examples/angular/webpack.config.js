const rspackConfig = require("./rspack.config");
delete rspackConfig.builtins;

// rspackConfig.optimization.concatenateModules = false;
// rspackConfig.optimization.mangleExports = false;
// rspackConfig.optimization.innerGraph = false;
module.exports = rspackConfig;
