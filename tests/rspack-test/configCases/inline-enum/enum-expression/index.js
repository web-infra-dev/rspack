import { E } from "./enum";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should inline for enum merging", () => {
  // START:A
  expect(typeof E.Dynamic).toBe("number");
  expect(E.Static).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`Dynamic).toBe("number")`)).toBe(true);
  expect(block.includes(`(/* inlined export .E.Static */ (1)).toBe(1)`)).toBe(true);
})
