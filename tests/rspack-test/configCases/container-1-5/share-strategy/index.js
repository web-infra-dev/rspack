const mfScope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		expect(App()).toBe("App rendered with [This is react 0.1.2]");
		const shareStrategy = mfScope.federation.initOptions.shareStrategy;
		expect(shareStrategy).toBe("loaded-first");

	});
});
