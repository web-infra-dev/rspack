module.exports = {
	noTests: true,
	afterExecute(options) {
		expect(options.cache.type).toBe("persistent");
		expect(options.experiments.newCache.codeGeneration).toBe(false);
		expect(options.plugins.find(plugin => plugin.name === "CountBuildsPlugin").builds).toBe(2);
	}
};
