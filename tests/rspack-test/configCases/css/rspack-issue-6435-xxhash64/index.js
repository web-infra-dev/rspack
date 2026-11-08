import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
	expect(classes["container-main"]).toBe("d8ad836b5119c8e8-container-main")
	expect(legacyClasses["legacy-main"]).toBe("_9721e623bccf86c6-legacy-main")
});
