import { E, Evaluatable } from "./enum";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should inline for enum expression", () => {
  // START:A
  expect(typeof E.Dynamic).toBe("number");
  expect(E.Static).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`Dynamic).toBe("number")`)).toBe(true);
  expect(block.includes(`((/* inlined export .E.Static */1)).toBe(1)`)).toBe(true);
})

it("should inline for enum evaluatable expression", () => {
  // START:B
  expect(Evaluatable.Neg).toBe(-1);
  expect(Evaluatable.Pos).toBe(1);
  expect(Evaluatable.Add).toBe(2);
  expect(Evaluatable.Ref).toBe(3);
  expect(Evaluatable.Ref2).toBe(4);
  expect(Evaluatable.Tail).toBe(5);
  // END:B
  const block = generated.match(/\/\/ START:B([\s\S]*)\/\/ END:B/)[1];
  expect(block.includes(`((/* inlined export .Evaluatable.Neg */-1)).toBe(-1)`)).toBe(true);
  expect(block.includes(`((/* inlined export .Evaluatable.Pos */1)).toBe(1)`)).toBe(true);
  expect(block.includes(`((/* inlined export .Evaluatable.Add */2)).toBe(2)`)).toBe(true);
  expect(block.includes(`((/* inlined export .Evaluatable.Ref */3)).toBe(3)`)).toBe(true);
  expect(block.includes(`((/* inlined export .Evaluatable.Ref2 */4)).toBe(4)`)).toBe(true);
  expect(block.includes(`((/* inlined export .Evaluatable.Tail */5)).toBe(5)`)).toBe(true);
})
