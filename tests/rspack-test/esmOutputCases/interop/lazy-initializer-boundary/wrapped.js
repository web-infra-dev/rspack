const child = require("./child.js");

eval("");
globalThis.__esmLazyEvaluationLog.push("wrapped");
module.exports = { value: child.value };
