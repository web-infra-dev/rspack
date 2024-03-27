const path = require("path");
const testBasePath = path.resolve(__dirname, "./tests");
const snapshotLegacyBasePath = path.resolve(
	__dirname,
	"../rspack/tests/__snapshots__"
);
const snapshotBasePath = path.resolve(__dirname, "./tests/__snapshots__");

const MIGRATED_CASES = [
	"Compiler.test.js",
	"Defaults.unittest.js",
	"Stats.test.js"
];

module.exports = {
	resolveSnapshotPath: (testPath, snapshotExtension) => {
		const snapshotPath = MIGRATED_CASES.includes(path.basename(testPath))
			? snapshotBasePath
			: snapshotLegacyBasePath;
		const relative = testPath.replace(testBasePath, "");
		return path.join(snapshotPath, relative) + snapshotExtension;
	},
	resolveTestPath: (snapshotFilePath, snapshotExtension) => {
		const snapshotPath = MIGRATED_CASES.includes(
			path.basename(snapshotFilePath).slice(0, -snapshotExtension.length)
		)
			? snapshotBasePath
			: snapshotLegacyBasePath;
		const relative = snapshotFilePath.replace(snapshotPath + path.sep, "");
		return relative.slice(0, -snapshotExtension.length);
	},
	testPathForConsistencyCheck: `tests${path.sep}example.test.js`
};
