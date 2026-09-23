import { readFileSync } from "fs";
import { join } from "path";
import { value } from "./style.module.css";

// The color fixtures are copied from webpack's css/reexport case.
it("should update repeated ICSS values when a transitive import changes", () => {
	const expected = WATCH_STEP === "0" ? "#e74c3c" : "#123456";
	const css = readFileSync(join(__dirname, "style.css"), "utf-8");
	for (const property of ["color", "border-color", "background"]) {
		expect(css).toContain(`${property}: ${expected};`);
	}
	expect(value).toBe(expected);
});
