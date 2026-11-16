const nodeOptions = process.env.NODE_OPTIONS;

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "package map of NODE_OPTIONS",
	options: () => {
		// The last option wins.
		process.env.NODE_OPTIONS =
			'--dummy="escaped \\" quote" --experimental-package-map=ignored.json --experimental-package-map="./package map.json"';
		return {
			resolve: {
				packageMap: "./configured-package-map.json"
			}
		};
	},
	diff: async (e, defaults) => {
		if (nodeOptions === undefined) {
			delete process.env.NODE_OPTIONS;
		} else {
			process.env.NODE_OPTIONS = nodeOptions;
		}
		// The defaults use the package map that Node.js was started with.
		await defaults.toMatchObject({
			resolve: {
				packageMap: expect.stringMatching(/[\\/]package map\.json$/)
			},
			resolveLoader: {
				packageMap: expect.stringMatching(/[\\/]package map\.json$/)
			}
		});
		await e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "packageMap": "<cwd>/package map.json",
			+     "packageMap": "./configured-package-map.json",
		`);
	}
};
