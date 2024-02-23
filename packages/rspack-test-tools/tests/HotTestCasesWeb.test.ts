import path from "path";
import { createHotCase } from "../src/case/hot";
import { describeByWalk } from "../src/helper";

const NAME = "HotTestCases";
const caseDir: string = path.resolve(__dirname, "../../rspack/tests/hotCases");
const distDir: string = path.resolve(
	__dirname,
	`../../rspack/tests/js/${NAME}`
);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createHotCase(name, src, dist, "web");
});
