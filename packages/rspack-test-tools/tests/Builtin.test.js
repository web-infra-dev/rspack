const path = require("path");
const { describeByWalk, createBuiltinCase } = require("..");

const NAME = "BuiltinCases";
const caseDir = path.resolve(__dirname, "./builtinCases");
const distDir = path.resolve(__dirname, "./js/builtins");

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createBuiltinCase(name, src, dist);
});
