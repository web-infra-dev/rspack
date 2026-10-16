import { value as positive } from "./node_modules/positive/lib/esm/index.js";
import { value as negative } from "./node_modules/negative/lib/esm/index.js";

it("invalidates inherited metadata and missing package dependencies", () => {
  expect(positive).toBe("positive");
  expect(negative).toBe("negative");
  const expected = [
    ["negative"],
    ["positive"],
    ["negative", "positive"],
    [],
    ["negative", "positive"],
    ["negative"],
  ][Number(WATCH_STEP)];
  expect((globalThis.__inheritedMarkers || []).sort()).toEqual(expected);
});
