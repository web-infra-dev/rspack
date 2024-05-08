import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${/* md4("./style.module.css") */ "ea850e6088d2566f677"}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${/* md4("./index.css") */ "a7200a43b5c2530b1414"}-legacy-main`)
});
