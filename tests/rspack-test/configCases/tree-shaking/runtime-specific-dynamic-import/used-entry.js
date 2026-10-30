import fs from "fs";
import path from "path";
import { loadEagerFeature, loadFeature } from "./module";

it("should retain the dynamic import in a runtime that uses it", async () => {
  const imported = await loadFeature();
  expect(imported.value).toBe("feature");
  expect(fs.existsSync(path.join(__dirname, "feature.js"))).toBe(true);
  expect((await loadEagerFeature()).value).toBe("EAGER_RUNTIME_FEATURE_MARKER");
});
