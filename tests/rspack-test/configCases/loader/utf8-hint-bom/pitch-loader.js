module.exports = function () {
  throw new Error("A pitch result should skip the producer normal function");
};

module.exports.pitch = require("./producer-loader");
