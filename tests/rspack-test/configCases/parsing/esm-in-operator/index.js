import * as m from "./m";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should tree shake b", () => {
  // START:A
  expect("a" in m).toBe(true);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`expect( true).toBe(true)`)).toBe(true);
  expect(m.usedB).toBe(false);
})
