async function getFallbacks() {
	const container = eval("require")("./remoteEntry.js").tree_shaking_shared_provide_only;
	return (await container.get("./fallbacks"))();
}

it("should omit federation from the ordinary provider-only entry", () => {
	expect(__webpack_require__.federation).toBeUndefined();
});

it("should generate tree shaking shared fallback for provide-only shared modules", async () => {
	const fallbacks = (await getFallbacks())["provided-only"];

	expect(fallbacks).toBeTruthy();
	expect(fallbacks.map(([, version]) => version)).toEqual(["1.0.0"]);

	const [entry, , globalName] = fallbacks[0];
	const container = eval("require")(`./${entry}`)[globalName];
	expect(container.get()().value).toBe("provided-only");
});

it("should use configured version for local provide-only shared fallbacks", async () => {
	const fallbacks = (await getFallbacks())["local-provided"];

	expect(fallbacks).toBeTruthy();
	expect(fallbacks.map(([, version]) => version)).toEqual(["2.3.4"]);

	const [entry, , globalName] = fallbacks[0];
	const container = eval("require")(`./${entry}`)[globalName];
	expect(container.get()().value).toBe("local-provided");
});
