const util = require("util");
const { Rspack } = require("@rspack/binding");

async function main() {
	const config = require("/Users/bytedance/Projects/rspack/examples/basic/test.config.js");
	const rspack = new Rspack(config);
	const result = await util.promisify(rspack.unsafe_build.bind(rspack))();
	console.log(result);
	rspack.unsafe_last_compilation(compilation => {
		console.log(compilation.getAssets());
	});
}

main();
