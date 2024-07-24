import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe(`${/* md4("./style.module.css") */ "d8ad836b5119c8e8"}-container-main`)
	expect(legacyClasses["legacy-main"]).toBe(`${/* md4("./legacy/index.css") */ "e623bccf86c6"}-legacy-main`)
});
