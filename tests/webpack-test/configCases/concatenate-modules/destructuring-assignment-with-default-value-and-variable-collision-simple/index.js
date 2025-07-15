import config from "./config";
import modA from "./module-a";

const { variableClash = "defaultValue" } = config;

it("renames a destructured assignment with default value correctly", () => {
	expect(modA).toBe("valueFromSomeFile");
	expect(variableClash).toBe("Correct value");
});
