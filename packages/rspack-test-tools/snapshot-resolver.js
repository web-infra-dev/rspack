const path = require("path");
const testBasePath = path.resolve(__dirname, "./tests");
const snapshotBasePath = path.resolve(
	__dirname,
	"../rspack/tests/__snapshots__"
);
module.exports = {
	resolveSnapshotPath: (testPath, snapshotExtension) => {
		const relative = testPath.replace(testBasePath, "");
		return path.join(snapshotBasePath, relative) + snapshotExtension;
	},
	resolveTestPath: (snapshotFilePath, snapshotExtension) => {
		const relative = snapshotFilePath.replace(snapshotBasePath + path.sep, "");
		return relative.slice(0, -snapshotExtension.length);
	},
	testPathForConsistencyCheck: `tests${path.sep}example.test.js`
};
