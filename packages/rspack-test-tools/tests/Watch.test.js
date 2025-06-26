const path = require("path");
const { describeByWalk, createWatchCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

describeByWalk(__filename, (name, src, dist) => {
	createWatchCase(name, src, dist, path.join(tempDir, name));
});
