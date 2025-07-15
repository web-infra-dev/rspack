import { a, y } from "pmodule";
import { log } from "pmodule/tracker";

it("should not evaluate a reexporting transitive module", function() {
	expect(a).toBe("a");
	expect(y).toBe("y");
	expect(log).toEqual(["a.js", "b.js"]);
});
