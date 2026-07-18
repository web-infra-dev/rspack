const initialRemovedFactory = __webpack_require__.m["./a.js"];
const readRemovedModule = require("./module");
const removedModule = readRemovedModule();
let valueDuringApply;

module.hot.accept("./module", () => {
	valueDuringApply = removedModule.value;
	removedModule.invalidate();
});

it("should finalize after recursively applying queued invalidations", async () => {
	await NEXT_HMR({
		ignoreUnaccepted: true,
		preserveDisposedModuleFactories: true
	});

	expect(valueDuringApply).toBe("a");
	expect(__webpack_require__.m["./a.js"]).toBe(initialRemovedFactory);
});
