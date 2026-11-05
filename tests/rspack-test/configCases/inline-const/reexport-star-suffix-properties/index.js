import { MASK, OTHER } from "./constants";

const generated = /** @type {string} */ (require("fs").readFileSync(__filename, "utf-8"));

it("should keep property access after an inlined export reached through export *", () => {
  // START:A
  expect(MASK.repeat(3)).toBe("•••");
  // END:A
  expect(OTHER).toBe(1);
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`(/* inlined export .MASK.repeat */"•").repeat(3)`)).toBe(true);
});
