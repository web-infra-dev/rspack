import { lib, lib1 } from "./lib";

it("`oneOf` should work", () => {
	// should run loader & loader2 and not run loader1
	expect(lib).toEqual("aabc");

	expect(lib1).toEqual("effg");
});
