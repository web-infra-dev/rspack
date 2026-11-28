let value;

// The harness runs this once the bundle settled, so it must be registered before the
// `await`: an `it` after it would never run in a bundle that settles too early.
it("should settle the bundle only after the async entry finished", () => {
	expect(value).toBe(42);
});

value = await new Promise((resolve) => {
	setTimeout(() => resolve(42), 50);
});

export {};
