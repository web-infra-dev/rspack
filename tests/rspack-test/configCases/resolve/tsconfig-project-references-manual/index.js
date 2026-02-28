import { a } from "./project_a/src/index";
import { b } from "./project_b/src/index";
// import { c } from './project_c/index'

it("should import the referenced alias", () => {
	expect(a).toEqual("a");
	expect(b).toEqual("b");
	// expect(c).toEqual("c");
});
