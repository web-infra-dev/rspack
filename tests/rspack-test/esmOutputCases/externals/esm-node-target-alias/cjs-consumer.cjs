// CJS require of "module" type external — should be downgraded to node-commonjs
const nodePath = require("node:fs");
exports.cjsResolve = nodePath.resolve;

// CJS require of "module-import" type external — should also be downgraded
const nodeUrl = require("node:url");
exports.cjsParse = nodeUrl.parse;
