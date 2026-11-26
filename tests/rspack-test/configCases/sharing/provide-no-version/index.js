it("should provide a shared package that declares no version", async () => {
	await import("no-version-pkg");

	await __webpack_init_sharing__("default");
	expect(Object.keys(__webpack_share_scopes__.default["no-version-pkg"])).toEqual(["0"]);
});
