const initialRemovedFactory = __webpack_require__.m["./a.js"];
const readRemovedModule = require("./module");

module.hot.accept("./module", () => {
	throw new Error("accept failed");
});

it("should finalize disposed factories when apply rejects", async () => {
	await expect(
		NEXT_HMR({ preserveDisposedModuleFactories: true })
	).rejects.toThrow("accept failed");

	expect(__webpack_require__.m["./a.js"]).not.toBe(initialRemovedFactory);
	const warn = console.warn;
	console.warn = () => {};
	try {
		expect(readRemovedModule).toThrow("RuntimeError: factory is undefined(./a.js)");
	} finally {
		console.warn = warn;
	}
});
