it("should reject a strict singleton consumer when the provider declares no version", async () => {
	await __webpack_init_sharing__("default");
	await expect(import("no-version-pkg")).rejects.toThrow(
		/Unsatisfied version 0 from .* of shared singleton module no-version-pkg \(required \^1\.0\.0\)/
	);
});
