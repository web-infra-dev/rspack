it("should inject version when use __rspack_unique_id__", () => {
	const version = require("../../../../package.json").version;
	expect(__rspack_unique_id__).toBe("bundler=rspack@" + version);
});
