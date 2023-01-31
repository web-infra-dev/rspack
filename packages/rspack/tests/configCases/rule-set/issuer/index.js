import { lib } from "./lib";

it("`issuer` should work", () => {
	// avoid being processed by loader
	const target = "cbaa".split("").reverse().join("");

	// should run loader and not run loader1
	expect(lib).toEqual(target);
});
