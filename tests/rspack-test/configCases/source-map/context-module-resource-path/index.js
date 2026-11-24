exports.load = name => require("./foo/" + name + ".js");
exports.context = require.context("./foo", true, /^\.\/(?:a|b)\.js$/);
