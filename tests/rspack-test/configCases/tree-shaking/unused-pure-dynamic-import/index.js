import fs from "fs";
import path from "path";
import { live } from "./module";

it("should not emit a chunk for a dynamic import in an unused pure declaration", () => {
  expect(live).toBe("live");
  expect(fs.existsSync(path.join(__dirname, "async.js"))).toBe(false);
  const eagerMarker = ["UNUSED", "EAGER", "DYNAMIC", "IMPORT", "MARKER"].join(
    "_",
  );
  expect(fs.readFileSync(__filename, "utf-8")).not.toContain(eagerMarker);
});
