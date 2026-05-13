import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${/* xxhash64("./style.module.css#container-main") */ "_9e2120a39eb3d3b7"}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${/* xxhash64("./legacy/index.css#legacy-main") */ "c1addbedadadb249"}-legacy-main`)
});
