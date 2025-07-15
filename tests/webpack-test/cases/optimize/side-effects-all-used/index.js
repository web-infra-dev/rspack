import def, { a, x, z } from "pmodule";
import { log } from "pmodule/tracker";

it("should evaluate all modules", function() {
	expect(def).toBe("def");
	expect(a).toBe("a");
	expect(x).toBe("x");
	expect(z).toBe("z");
	expect(log).toEqual(["a.js", "b.js", "c.js", "index.js"]);
});
