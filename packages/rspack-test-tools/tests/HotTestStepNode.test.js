const path = require("path");
const { describeByWalk, createHotStepCase } = require("..");

const NAME = "HotStepTestCasesNode";
const caseDir = path.resolve(__dirname, "../../rspack/tests/hotCases");
const distDir = path.resolve(
	__dirname,
	`../../rspack/tests/js/hot-step-cases-node`
);

describeByWalk(
	NAME,
	caseDir,
	distDir,
	(name, src, dist) => {
		createHotStepCase(name, src, dist, "async-node");
	},
	{
		cat: /status/,
		case: /accept/
	}
);
