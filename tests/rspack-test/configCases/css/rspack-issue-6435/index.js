import * as classes from "./style.module.css";
import legacyClasses from "./legacy/index.css";

it("should have consistent hash", () => {
  const suffix = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK ? "-rspack" : "";
  expect(classes["container-main"]).toMatch(/^_?[a-f0-9]{20}-container-main$/);
  expect(legacyClasses["legacy-main"]).toMatch(/^_?[a-f0-9]{20}-legacy-main$/);
  expect(classes["container-main"]).toMatchFileSnapshotSync(`${__SNAPSHOT__}/container-main${suffix}.txt`);
  expect(legacyClasses["legacy-main"]).toMatchFileSnapshotSync(`${__SNAPSHOT__}/legacy-main${suffix}.txt`);
});
