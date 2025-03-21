// var supportsWebAssembly = require("../../../helpers/supportsWebAssembly");
// var supportsResponse = require("../../../helpers/supportsResponse");
// module.exports = function (config) {
//        return supportsWebAssembly() && supportsResponse();
// };

// For the selected environment is no default ESM chunk format available
module.exports = () => false;
