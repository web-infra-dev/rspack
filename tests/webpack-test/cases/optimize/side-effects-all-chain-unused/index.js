import { a } from "pmodule";
import { log } from "pmodule/tracker";

it("should not evaluate a chain of modules", function() {
	expect(a).toBe("a");
	expect(log).toEqual(["a.js"]);
});
