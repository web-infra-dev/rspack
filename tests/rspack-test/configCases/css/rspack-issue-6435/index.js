import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${/* md4("./style.module.css#container-main") */ "_2cc004eb30c213462c0e"}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${/* md4("./legacy/index.css#legacy-main") */ "_4f4c57e79732c3a9021a"}-legacy-main`)
});
