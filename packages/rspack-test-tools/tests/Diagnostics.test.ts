import path from "path";
import { createDiagnosticCase } from "../src/case/diagnostic";
import { describeByWalk } from "../src/helper";

const NAME = "HotTestCases";
const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/diagnostics"
);

describeByWalk(NAME, caseDir, "", (name, src, dist) => {
	createDiagnosticCase(
		name,
		src,
		path.join(src, "dist"),
		path.resolve(__dirname, "../../rspack")
	);
});
