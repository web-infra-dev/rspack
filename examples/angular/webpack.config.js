const rspackConfig = require("./rspack.config");
delete rspackConfig.builtins;

rspackConfig.optimization.concatenateModules = false;
rspackConfig.optimization.mangleExports = false;
rspackConfig.optimization.innerGraph = false;
// rspackConfig.optimization.sideEffects = 'flag';
module.exports = rspackConfig;
