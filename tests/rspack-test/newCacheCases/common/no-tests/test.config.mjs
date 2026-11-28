export default {
	noTests: true,
	afterExecute(options) {
		// User cache configuration wins over the suite defaults.
		expect(options.cache).toBe(false);
		expect(options.plugins.find(plugin => plugin.name === "CountBuildsPlugin").builds).toBe(2);
	}
};
