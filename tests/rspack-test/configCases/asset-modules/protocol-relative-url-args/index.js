it("should walk ignored protocol-relative URL arguments", async () => {
	globalThis.protocolRelativeUrlSideEffect = false;
	new URL(
		"//cdn.example.com/a.png",
		import.meta.url,
		(globalThis.protocolRelativeUrlPromise = import("./side"))
	);
	await globalThis.protocolRelativeUrlPromise;
	expect(globalThis.protocolRelativeUrlSideEffect).toBe(true);
});
