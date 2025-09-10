import mappedValue from "exports-field/dist/main";
import entryValue from "exports-field";

it("conditionNames should works", () => {
	expect(mappedValue).toBe("lib/lib2/main");
	expect(entryValue).toBe("x");
	const pkgValue = require("exports-field/package.json");
	expect(pkgValue.name).toBe("exports-field");
});
