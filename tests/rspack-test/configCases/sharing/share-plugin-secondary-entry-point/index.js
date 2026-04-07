it("should provide a secondary entry point using the parent package version", async () => {
	const styles = await import("@scope/pkg/styles");
	expect(styles).toEqual(
		expect.objectContaining({
			default: "pkg-styles"
		})
	);

	await __webpack_init_sharing__("default");
	expect(Object.keys(__webpack_share_scopes__.default)).toContain("@scope/pkg/styles");
	expect(Object.keys(__webpack_share_scopes__.default["@scope/pkg/styles"])).toContain("1.2.3");
});

it("should provide the root package normally", async () => {
	const pkg = await import("@scope/pkg");
	expect(pkg).toEqual(
		expect.objectContaining({
			default: "pkg-root"
		})
	);
});
