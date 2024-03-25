const path = require("path");
const { describeByWalk, createWatchCase } = require("..");

const NAME = "WatchTestCases";
const caseDir = path.resolve(__dirname, "../../rspack/tests/watchCases");
const distDir = path.resolve(__dirname, `../../rspack/tests/js/${NAME}`);
const tempDir = path.resolve(__dirname, `../../rspack/tests/js/${NAME}-src`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createWatchCase(
		name,
		src,
		dist,
		path.resolve(tempDir, path.relative(distDir, dist))
	);
});
