import { lib0, lib1, lib2, lib3 } from "./lib0";

it("`issuer` should work", () => {
	expect(lib0).toEqual("lib0(#)loader0");
	expect(lib1).toEqual("lib1(#)loader0loader1");
	expect(lib2).toEqual("lib2(#)loader0loader1loader2loader3");
	expect(lib3).toEqual("lib3(#)loader0loader1loader3");
});
