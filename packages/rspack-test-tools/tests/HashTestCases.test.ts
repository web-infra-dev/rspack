import path from "path";
import { describeByWalk } from "../src/helper";
import { createConfigCase } from "../src/case/config";

const NAME = "HashTestCases";
const caseDir: string = path.resolve(__dirname, "../../rspack/tests/hashCases");

describeByWalk(NAME, caseDir, "", (name, src) => {
	createConfigCase(name, src, path.join(src, "dist"));
});
