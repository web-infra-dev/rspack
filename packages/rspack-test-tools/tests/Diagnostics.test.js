const path = require("path");
const { describeByWalk, createDiagnosticCase } = require("..");

const NAME = "HotTestCases";
const caseDir = path.resolve(__dirname, "./diagnostics");

describeByWalk(NAME, caseDir, "", (name, src, dist) => {
	createDiagnosticCase(name, src, path.join(src, "dist"));
});
