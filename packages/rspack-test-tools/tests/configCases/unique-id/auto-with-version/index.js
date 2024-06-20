it("should inject version when use __rspack_unique_id__", () => {
	expect(__rspack_unique_id__).toBe("bundler=rspack@" + __rspack_version__);
});
