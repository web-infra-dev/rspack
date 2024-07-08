import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${/* md4("./style.module.css") */ "ea850e6088d2566f677"}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${/* md4("./legacy/index.css") */ "c15d43fe622e87bbf5d"}-legacy-main`)
});
