const execArgv = process.execArgv;

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "package map of the Node.js command line",
	options: () => {
		// The last option wins, and the command line takes precedence over NODE_OPTIONS.
		process.execArgv = [
			...execArgv,
			"--experimental-package-map=ignored.json",
			"--experimental-package-map",
			"./package-map.json"
		];
		return {
			resolveLoader: {
				packageMap: false
			}
		};
	},
	diff: async (e, defaults) => {
		process.execArgv = execArgv;
		// The defaults use the package map that Node.js was started with.
		await defaults.toMatchObject({
			resolve: {
				packageMap: expect.stringMatching(/[\\/]package-map\.json$/)
			},
			resolveLoader: {
				packageMap: expect.stringMatching(/[\\/]package-map\.json$/)
			}
		});
		await e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "packageMap": "<cwd>/package-map.json",
			+     "packageMap": false,
		`);
	}
};
