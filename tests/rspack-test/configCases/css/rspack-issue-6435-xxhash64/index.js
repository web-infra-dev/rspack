import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(classes["container-main"]).toBe("b8080a47f909c69c-container-main")
		expect(legacyClasses["legacy-main"]).toBe("_7e25f4920b87223b-legacy-main")
	} else {
		expect(classes["container-main"]).toBe("_55c63d4f54dc0364-container-main")
		expect(legacyClasses["legacy-main"]).toBe("_2064ffe458f64a41-legacy-main")
	}
});
