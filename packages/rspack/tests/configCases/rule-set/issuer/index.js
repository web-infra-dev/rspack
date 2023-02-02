import { lib, lib1 } from "./lib";

it("`issuer` should work", () => {
	// should run loader and not run loader1
	expect(lib).toEqual("aabc");

	// should run loader and loader1
	expect(lib1).toEqual("aabcc");
});
