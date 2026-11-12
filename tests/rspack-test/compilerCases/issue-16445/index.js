import { check } from "./noop-wrapper.js";

export function run(value) {
	let predicateCalls = 0;
	check(() => {
		predicateCalls++;
		throw new Error("MINIMAL_CALLBACK_BODY");
	});
	return { value, predicateCalls };
}

export function runFactory() {
	let factoryCalls = 0;
	let predicateCalls = 0;
	function makeCallback() {
		factoryCalls++;
		return () => predicateCalls++;
	}
	check(makeCallback());
	return { factoryCalls, predicateCalls };
}
