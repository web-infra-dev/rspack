import { value } from "./dep";
import { value as aValue } from "./a";
import { value as bValue } from "./b";
import { value as throwingValue } from "./throwing";
import { value as laterValue } from "./later";

let acceptedDep;
let acceptedDepAgain;
let acceptedArrays = [];
let acceptedDuplicateArrays = [];
let acceptedErrorArray;
let webpackAcceptedDependencies;
let webpackAcceptCalls = 0;

if (module.hot) {
	module.hot.accept("./dep", outdatedDependencies => {
		webpackAcceptCalls += 1;
		webpackAcceptedDependencies = outdatedDependencies;
	});
}

if (import.meta.hot) {
	import.meta.hot.accept("./dep", mod => {
		acceptedDep = mod;
	});
	import.meta.hot.accept("./dep", mod => {
		acceptedDepAgain = mod;
	});
	import.meta.hot.accept(["./a", "./b"], mods => {
		acceptedArrays.push(mods);
	});
	import.meta.hot.accept(["./a", "./a"], mods => {
		acceptedDuplicateArrays.push(mods);
	});
	import.meta.hot.accept(["./throwing", "./later"], mods => {
		acceptedErrorArray = mods;
	});
	import.meta.hot.accept("./later");
}

it("continues dependency refreshes and callbacks after an update error", async () => {
	expect(value).toBe(1);
	expect(aValue).toBe("a1");
	expect(bValue).toBe("b1");
	expect(throwingValue).toBe("throwing1");
	expect(laterValue).toBe("later1");

	let applyError;
	try {
		await NEXT_HMR();
	} catch (error) {
		applyError = error;
	}

	expect(applyError.message).toBe("throwing dependency update");
	expect(acceptedDep.value).toBe(2);
	expect(acceptedDepAgain.value).toBe(2);
	expect(webpackAcceptCalls).toBe(1);
	expect(webpackAcceptedDependencies).toContain("./dep.js");
	expect(acceptedArrays.map(mods => mods.map(mod => mod && mod.value))).toEqual([
		["a2", undefined],
		[undefined, "b2"]
	]);
	expect(
		acceptedDuplicateArrays.map(mods => mods.map(mod => mod && mod.value))
	).toEqual([["a2", "a2"]]);
	expect(acceptedErrorArray.map(mod => mod && mod.value)).toEqual([
		undefined,
		"later2"
	]);
	expect(value).toBe(2);
	expect(aValue).toBe("a2");
	expect(bValue).toBe("b2");
	expect(laterValue).toBe("later2");
});
