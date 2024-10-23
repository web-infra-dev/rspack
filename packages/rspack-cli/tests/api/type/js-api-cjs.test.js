const assert = require("node:assert");

describe("js-api-type should be correct when requiring from @rspack/core", () => {
	it("cjs require", async () => {
		const rspack = require("@rspack/core");
		const compiler = rspack({});
		assert(rspack.BannerPlugin);
		assert(typeof rspack === "function");
		assert(compiler);
	});

	it("cjs namedRequire", async () => {
		const { rspack } = require("@rspack/core");
		const compiler = rspack({});
		assert(rspack.BannerPlugin);
		assert(typeof rspack === "function");
		assert(compiler);
	});
});
