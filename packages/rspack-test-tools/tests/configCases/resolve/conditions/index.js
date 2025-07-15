import entryValue from "exports-field";
import mappedValue from "exports-field/dist/main";

it("conditionNames should works", () => {
	expect(mappedValue).toBe("lib/lib2/main");
	expect(entryValue).toBe("x");
	const pkgValue = require("exports-field/package.json");
	expect(pkgValue.name).toBe("exports-field");
});
