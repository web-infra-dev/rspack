const path = require("path");
const { describeByWalk, createDiagnosticCase } = require("..");

const NAME = "HotTestCases";
const caseDir = path.resolve(__dirname, "../../rspack/tests/diagnostics");

describeByWalk(NAME, caseDir, "", (name, src, dist) => {
	createDiagnosticCase(
		name,
		src,
		path.join(src, "dist"),
		path.resolve(__dirname, "../../rspack")
	);
});
