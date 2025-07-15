import n from "nmodule";
import { log as nlog } from "nmodule/tracker";
import p from "pmodule";
import { log as plog } from "pmodule/tracker";

it("should be able to override side effects", function() {
	expect(p).toBe("def");
	expect(n).toBe("def");
	expect(plog).toEqual(["a.js", "b.js", "c.js", "index.js"]);
	expect(nlog).toEqual(["index.js"]);
});
