it("should walk ignored protocol-relative URL arguments", async () => {
	const exactUrl = new URL("//cdn.example.com/exact.png", import.meta.url);
	expect(exactUrl.href).toBe("file://cdn.example.com/exact.png");

	globalThis.protocolRelativeUrlSideEffect = false;
	new URL(
		"//cdn.example.com/a.png",
		import.meta.url,
		(globalThis.protocolRelativeUrlPromise = import("./side"))
	);
	await globalThis.protocolRelativeUrlPromise;
	expect(globalThis.protocolRelativeUrlSideEffect).toBe(true);
});
