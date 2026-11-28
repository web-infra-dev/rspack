it("should consume an eager provider while the remote is initializing", async () => {
	const initialization = __webpack_init_sharing__("default");
	expect(typeof initialization.then).toBe("function");
	expect(require("eager-sync")).toBe("host");
	await initialization;
	expect((await import("remote/module")).default).toBe("remote");
});

it("should select an eager provider for a synchronous eager consume", async () => {
	await __webpack_init_sharing__("default");
	expect(require("eager-sync")).toBe("host");
});

it("should select an eager provider for an asynchronous eager consume", async () => {
	await __webpack_init_sharing__("default");
	expect((await import("eager-async")).default).toBe("host");
});

it("should allow an asynchronous non-eager consume to select a non-eager provider", async () => {
	await __webpack_init_sharing__("default");
	expect((await import("non-eager-async")).default).toBe("remote");
});
