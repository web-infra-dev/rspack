const m = require("./m");

module.exports.load = () => require("./m");
module.exports.initial = m;
module.exports.getModule = () => module;
