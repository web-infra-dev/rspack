import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe("_55c63d4f54dc0364-container-main")
	expect(legacyClasses["legacy-main"]).toBe("_2064ffe458f64a41-legacy-main")
});
