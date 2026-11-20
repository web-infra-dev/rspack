const nodeOptions = process.env.NODE_OPTIONS;

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "no package map in NODE_OPTIONS",
	options: () => {
		// Node.js refuses to start with an unterminated string, and a later empty path unsets the option.
		process.env.NODE_OPTIONS =
			'--experimental-package-map ./package-map.json --experimental-package-map= --title="unterminated';
		return {};
	},
	diff: async (e, defaults) => {
		process.env.NODE_OPTIONS =
			"--experimental-package-map ./package-map.json --experimental-package-map=";
		const { config } = await import("@rspack/core");
		const emptyPath = config.getNormalizedRspackOptions({});
		config.applyRspackOptionsDefaults(emptyPath);
		process.env.NODE_OPTIONS = "--trace-warnings --experimental-package-map ./package-map.json";
		const separateValue = config.getNormalizedRspackOptions({});
		config.applyRspackOptionsDefaults(separateValue);
		if (nodeOptions === undefined) {
			delete process.env.NODE_OPTIONS;
		} else {
			process.env.NODE_OPTIONS = nodeOptions;
		}

		await defaults.not.toHaveProperty("resolve.packageMap");
		await defaults.not.toHaveProperty("resolveLoader.packageMap");
		expect(emptyPath.resolve).not.toHaveProperty("packageMap");
		// The path can be a separate argument.
		expect(separateValue.resolve.packageMap).toMatch(/[\\/]package-map\.json$/);
		await e.toMatchInlineSnapshot(`Compared values have no visual difference.`);
	}
};
