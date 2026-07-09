import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(classes["container-main"]).toBe("_87df500bb85ed2e1b1f0-container-main")
    expect(legacyClasses["legacy-main"]).toBe("_8f645d48bbc8d62b15e8-legacy-main")
  } else {
    expect(classes["container-main"]).toBe("_467c4885db406636e4bf-container-main")
    expect(legacyClasses["legacy-main"]).toBe("_472dae718ba45ef203c9-legacy-main")
  }
});
