let value = require("./value");

it("should resolve an entry filename containing the full hash", async () => {
	expect(value).toBe("a");
	expect((await import("./async")).value).toBe("async");
	expect(__webpack_require__.u("main")).toMatch(/^main\.[a-f0-9]+\.js$/);
	await NEXT_HMR();
	expect(value).toBe("b");
});

module.hot.accept("./value", () => {
	value = require("./value");
});
