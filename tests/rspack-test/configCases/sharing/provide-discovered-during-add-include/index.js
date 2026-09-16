it("should finish the build when a provider is first discovered while building an included provider", async () => {
	const provided =
		__webpack_require__.initializeSharingData.scopeToSharingDataMapping.default;
	const entry = provided.find((item) => item.name === "unused-provider");
	expect(entry).toBeDefined();
	expect(entry.version).toBe("1.0.0");
	const factory = await entry.factory();
	expect(factory()).toBe("package");
});
