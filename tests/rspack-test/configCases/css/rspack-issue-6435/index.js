import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(classes["container-main"]).toBe("c4e49a91299807229fc9-container-main")
    expect(legacyClasses["legacy-main"]).toBe("_8f645d48bbc8d62b15e8-legacy-main")
  } else {
    expect(classes["container-main"]).toBe("e49dcf8a2397e06e4127-container-main")
    expect(legacyClasses["legacy-main"]).toBe("_472dae718ba45ef203c9-legacy-main")
  }
});
