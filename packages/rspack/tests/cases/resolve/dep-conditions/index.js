import foo from "./foo";
import dev from "dev-condition";
import v1 from "dep-condition";
const v2 = require("dep-condition");

it("should make different modules for resolve", function () {
	expect(foo).toBe("foo");
	expect(dev).toBe("development");
	expect(v1).toBe("import");
	expect(v2).toBe("require");
});
