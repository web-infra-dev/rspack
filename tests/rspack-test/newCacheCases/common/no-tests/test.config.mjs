export default {
	noTests: true,
	afterExecute(options) {
		expect(options.cache.type).toBe("persistent");
		expect(options.plugins.find(plugin => plugin.name === "CountBuildsPlugin").builds).toBe(2);
	}
};
