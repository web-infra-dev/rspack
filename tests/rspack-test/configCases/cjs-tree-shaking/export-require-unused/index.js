import { useMemo } from "./reexport";

it("should import by export require", () => {
	expect(useMemo).toBe("useMemo");
});

it("should flag other unused items with __webpack_unused_export__", () => {
	const mainFile = require("fs").readFileSync(__filename, "utf-8");
	const flag = "__webpack_unused_export__";
	for (let i of ["useState", "useEffect"]) {
		expect(mainFile.includes(`${flag} = "${i}"`)).toBeTruthy();
	}
});
