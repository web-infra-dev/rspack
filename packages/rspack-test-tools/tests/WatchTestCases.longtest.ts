import path from "path";
import { createWatchCase } from "../src/case/watch";
import { describeByWalk } from "../src/helper";

const NAME = "WatchTestCases";
const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/watchCases"
);
const distDir: string = path.resolve(
	__dirname,
	`../../rspack/tests/js/${NAME}`
);
const tempDir: string = path.resolve(
	__dirname,
	`../../rspack/tests/js/${NAME}-src`
);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createWatchCase(
		name,
		src,
		dist,
		path.resolve(tempDir, path.relative(distDir, dist))
	);
});
