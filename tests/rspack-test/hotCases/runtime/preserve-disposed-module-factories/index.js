const initialRemovedFactory = __webpack_require__.m["./a.js"];
require("./module");
let removedModuleId = "./a.js";
let factoryDuringApply;
let factoryAtIdle;

module.hot.accept("./module", () => {
	factoryDuringApply = __webpack_require__.m[removedModuleId];
});

const checkDisposedFactoryAtIdle = (status) => {
	if (status !== "idle") return;
	factoryAtIdle = __webpack_require__.m[removedModuleId];
};
module.hot.addStatusHandler(checkDisposedFactoryAtIdle);

it("should preserve disposed factories only for the requested apply transaction", async () => {
	await NEXT_HMR({ preserveDisposedModuleFactories: true });

	expect(factoryDuringApply).toBe(initialRemovedFactory);
	expect(factoryAtIdle).not.toBe(initialRemovedFactory);
	expect(__webpack_require__.m["./a.js"]).toBe(factoryAtIdle);

	module.hot.removeStatusHandler(checkDisposedFactoryAtIdle);
	require("./module");
	removedModuleId = "./b.js";
	const secondRemovedFactory = __webpack_require__.m[removedModuleId];
	await NEXT_HMR();

	expect(factoryDuringApply).not.toBe(secondRemovedFactory);
	expect(__webpack_require__.m[removedModuleId]).toBe(factoryDuringApply);
});
