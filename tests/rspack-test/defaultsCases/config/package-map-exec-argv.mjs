const execArgv = process.execArgv;
const nodeOptions = process.env.NODE_OPTIONS;

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "package map of the Node.js command line",
	options: () => {
		// The command line takes precedence over NODE_OPTIONS, and its last option wins.
		process.env.NODE_OPTIONS = "--experimental-package-map=./env-package-map.json";
		process.execArgv = [
			...execArgv,
			"--experimental-package-map=ignored.json",
			// Node.js accepts underscores in place of the dashes of an option name.
			"--experimental_package_map",
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
		if (nodeOptions === undefined) {
			delete process.env.NODE_OPTIONS;
		} else {
			process.env.NODE_OPTIONS = nodeOptions;
		}
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
