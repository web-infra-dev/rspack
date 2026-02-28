const moduleValue = require("./module");
const external = require("external");
import referencer from "./referencer";

it("should keep the module hash when usage changes", async () => {
	expect(moduleValue).toBe("module");
	expect(external).toBe("external");
	expect(referencer).toBe(42);
	await NEXT_HMR();
	expect(referencer).toBe("undefined undefined");
});

module.hot.accept("./referencer");
