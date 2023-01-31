import { lib } from "./lib";

it("`issuer` should work", () => {
	const target = "cbaa".split("").reverse().join("");
	expect(lib).toEqual(target);
});
