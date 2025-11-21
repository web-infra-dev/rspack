function deepReplace(obj) {
	if (typeof obj === "object" && obj !== null) {
		for (const key in obj) {
			if (Object.prototype.hasOwnProperty.call(obj, key)) {
				if (typeof obj[key] === "number" && key === "runtime") {
					obj[key] = "xxx";
				} else if (key === "hash") {
					obj[key] = "xxxxxxxxxxxxxxxx";
				} else if (typeof obj[key] === "object") {
					deepReplace(obj[key]);
				}
			}
		}
	}
}

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should output the chunks",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/chunk-b"
		};
	},
	async check(stats) {
		const statsChunks = stats?.toJson({
			chunks: true,
			timings: false,
			builtAt: false,
			version: false,
			modulesSpace: 3
		}).chunks;

		deepReplace(statsChunks);

		const string = stats.toString({
			chunks: true,
			timings: false,
			builtAt: false,
			version: false,
			modulesSpace: 3
		}).replace(/[\d.]+ (KiB|bytes)/g, "X.X $1");
		expect(string).toContain(`chunk (runtime: main) chunkB.js (chunkB) X.X bytes [rendered]`);
		expect(string).toContain(`./fixtures/b.js X.X bytes [built] [code generated]`);
		expect(string).toContain(`chunk (runtime: main) main.js (main) X.X bytes (javascript) X.X KiB (runtime) [entry] [rendered]`);
		expect(string).toContain(`./fixtures/chunk-b.js X.X bytes [built] [code generated]`);
	}
};
