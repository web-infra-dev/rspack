// var supportsWebAssembly = require("../../../helpers/supportsWebAssembly");
// module.exports = function (config) {
//        return supportsWebAssembly();
// };

// For the selected environment is no default ESM chunk format available
module.exports = () => false;
