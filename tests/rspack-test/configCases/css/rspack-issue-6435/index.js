import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  expect(classes["container-main"]).toBe("_467c4885db406636e4bf-container-main")
  expect(legacyClasses["legacy-main"]).toBe("_472dae718ba45ef203c9-legacy-main")
});
