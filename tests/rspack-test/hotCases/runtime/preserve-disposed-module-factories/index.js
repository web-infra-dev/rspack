const initialRemovedFactory = __webpack_require__.m["./a.js"];
const readRemovedModule = require("./module");
const getChildParents = require("./child").getParents;
let removedModuleId = "./a.js";
let factoryDuringApply;
let factoryAtIdle;
let valueDuringApply;
let childParentsDuringApply;
let readChildParentsAfterApply;

module.hot.accept("./module", () => {
	factoryDuringApply = __webpack_require__.m[removedModuleId];
	if (removedModuleId === "./a.js") {
		valueDuringApply = readRemovedModule();
		childParentsDuringApply = getChildParents();
		readChildParentsAfterApply = __webpack_require__("./a.js").readChildParents;
	}
});

const checkDisposedFactoryAtIdle = (status) => {
	if (status !== "idle") return;
	factoryAtIdle = __webpack_require__.m[removedModuleId];
};
module.hot.addStatusHandler(checkDisposedFactoryAtIdle);

it("should preserve disposed factories only for the requested apply transaction", async () => {
	await NEXT_HMR({ preserveDisposedModuleFactories: true });

	expect(factoryDuringApply).toBe(initialRemovedFactory);
	expect(valueDuringApply).toBe("a");
	expect(childParentsDuringApply).toContain("./a.js");
	expect(getChildParents()).not.toContain("./a.js");
	expect(factoryAtIdle).not.toBe(initialRemovedFactory);
	expect(__webpack_require__.m["./a.js"]).toBe(factoryAtIdle);
	const warnings = [];
	const warn = console.warn;
	console.warn = warning => warnings.push(warning);
	try {
		expect(readChildParentsAfterApply()).not.toContain("./a.js");
	} finally {
		console.warn = warn;
	}
	expect(warnings).toEqual([
		"[HMR] unexpected require(./child.js) from disposed module ./a.js"
	]);
	console.warn = () => {};
	try {
		expect(readRemovedModule).toThrow("RuntimeError: factory is undefined(./a.js)");
	} finally {
		console.warn = warn;
	}

	module.hot.removeStatusHandler(checkDisposedFactoryAtIdle);
	require("./module");
	removedModuleId = "./b.js";
	const secondRemovedFactory = __webpack_require__.m[removedModuleId];
	await NEXT_HMR();

	expect(factoryDuringApply).not.toBe(secondRemovedFactory);
	expect(__webpack_require__.m[removedModuleId]).toBe(factoryDuringApply);
});
