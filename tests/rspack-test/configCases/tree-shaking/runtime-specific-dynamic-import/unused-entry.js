import fs from "fs";
import { live } from "./module";

it("should skip the dynamic import in a runtime that does not use it", () => {
  expect(live).toBe("live");
  const eagerMarker = ["EAGER", "RUNTIME", "FEATURE", "MARKER"].join("_");
  expect(fs.readFileSync(__filename, "utf-8")).not.toContain(eagerMarker);
});
