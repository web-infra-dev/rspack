import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(classes["container-main"]).toBe("c4e49a91299807229fc9-container-main")
    expect(legacyClasses["legacy-main"]).toBe("fc6b8ed3d0ff87063b2f-legacy-main")
  } else {
    expect(classes["container-main"]).toBe("e49dcf8a2397e06e4127-container-main")
    expect(legacyClasses["legacy-main"]).toBe("_70bba980f1e761daa7f8-legacy-main")
  }
});
