import path from "path";
import { describeByWalk } from "../src/helper";
import { createNormalCase } from "../src/case/normal";

const NAME = "TestCases";
const caseDir: string = path.resolve(__dirname, "../../rspack/tests/cases");
const distDir: string = path.resolve(__dirname, `../../rspack/tests/js/normal`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createNormalCase(name, src, dist, path.resolve(__dirname, "../../rspack"));
});
