it("should inject version when use __rspack_version__", () => {
	expect(__rspack_version__).toBe(require("../../../../package.json").version);
});
