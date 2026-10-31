import value from "./holder";

it("should install a removed module again from a chunk loaded after it is restored", async () => {
	expect(value).toBe("restored");
	// Removes ./restored.js from the build, leaving the disposed module placeholder.
	await NEXT_HMR();
	expect(value).toBe("removed");
	// Restores ./restored.js only in the lazy chunk, which has not been loaded yet.
	await NEXT_HMR();
	expect((await import("./lazy")).default).toBe("restored");
});

module.hot.accept("./holder");
