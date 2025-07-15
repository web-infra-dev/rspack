import { a, z } from "pmodule";
import { log } from "pmodule/tracker";

it("should not evaluate an immediate module", function() {
	expect(a).toBe("a");
	expect(z).toBe("z");
	expect(log).toEqual(["a.js", "c.js"]);
});
