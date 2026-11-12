import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  expect(classes["container-main"]).toBe("_7ea850e6088d2566f677-container-main")
  expect(legacyClasses["legacy-main"]).toBe("_2c15d43fe622e87bbf5d-legacy-main")
});
