it("should keep remotes lazy when consuming a loaded-first share from a chunk", async () => {
	try {
		const { default: App } = await import("./App");
		expect(App()).toBe("App rendered with [This is react 0.1.2]");
		const shareStrategy = __webpack_require__.federation.initOptions.shareStrategy;
		expect(shareStrategy).toBe("loaded-first");
		expect(globalThis.__loadedFirstRemoteLoads || 0).toBe(0);
		expect((await import("lazy/module")).default).toBe("remote");
		expect(globalThis.__loadedFirstRemoteLoads).toBe(1);
	} finally {
		delete globalThis.__loadedFirstRemoteLoads;
	}
});
