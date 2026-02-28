it("should inject runtime plugin with params", async () => {
	return import("./App").then(({ default: App }) => {
		expect(App()).toBe("App rendered with [This is react 0.2.1]");
		const runtimePlugin = __webpack_require__.federation.initOptions.plugins[0];
		expect(runtimePlugin.getParams()).toMatchObject({
			'custom-params': {
				msg: 'custom-params',
			}
		});
	});
});
