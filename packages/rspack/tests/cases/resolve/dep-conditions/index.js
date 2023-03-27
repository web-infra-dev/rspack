import prod from "prod-condition";
import v1 from "dep-condition";
const v2 = require("dep-condition");

it("should make different modules for resolve", function () {
	expect(prod).toBe("production");
	expect(v1).toBe("import");
	expect(v2).toBe("require");
});
