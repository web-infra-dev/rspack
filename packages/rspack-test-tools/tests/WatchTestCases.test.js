const path = require("path");
const { describeByWalk, createWatchCase } = require("../dist");

const NAME = "WatchTestCases";
const caseDir = path.resolve(__dirname, "./watchCases");
const distDir = path.resolve(__dirname, `./js/${NAME}`);
const tempDir = path.resolve(__dirname, `./js/${NAME}-src`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createWatchCase(
		name,
		src,
		dist,
		path.resolve(tempDir, path.relative(distDir, dist))
	);
});
