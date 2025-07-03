const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

const isWasm = Boolean(process.env.WASM)

if (!isWasm) {
	describeByWalk(__filename, (name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	}, {
		source: path.join(__dirname, `./watchCases`),
	});
} else {
	it.skip("Should skip in wasm platform, because notify's dependency filetime is not support time", async () => { })
}

