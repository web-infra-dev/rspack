// const supportsWorker = require("../../../helpers/supportsWorker");
// module.exports = function (config) {
// 	return supportsWorker();
// };

// For the selected environment is no default ESM chunk format available
module.exports = () => false;
