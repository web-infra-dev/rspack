it("should inject version when use __rspack_version__", () => {
	expect(__rspack_version__).toBe(require("@rspack/core/package.json").version);
});
