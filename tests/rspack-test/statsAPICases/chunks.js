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

defineStatsAPICase(Utils.basename(__filename), {
	description: "should output the chunks",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./chunk-b"
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
		});
		expect(string).toContain(`chunk (runtime: main) chunkB.js (chunkB) 94 bytes [rendered]`);
		expect(string).toContain(`./b.js 94 bytes [built] [code generated]`);
		expect(string).toContain(`chunk (runtime: main) main.js (main) 85 bytes (javascript) 8.33 KiB (runtime) [entry] [rendered]`);
		expect(string).toContain(`./chunk-b.js 85 bytes [built] [code generated]`);
	}
});
