import { E } from "./enum";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should inline for enum when builtin:swc-loader is not the last loader", () => {
  // START:A
  expect(E.A).toBe(0);
  expect(E.B).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`((/* inlined export .E.A */0)).toBe(0)`)).toBe(true);
  expect(block.includes(`((/* inlined export .E.B */1)).toBe(1)`)).toBe(true);
})
