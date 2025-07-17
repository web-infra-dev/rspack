const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

const ignores = ["dynamic-entries"];

describeByWalk(
	__filename,
	(name, src, dist) => {
		if (ignores.some(ignore => name.includes(ignore))) {
			// Skip ignored cases
			console.log(`Skipping test case: ${name}`);
			return;
		}

		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(__dirname, `./watchCases`)
	}
);
