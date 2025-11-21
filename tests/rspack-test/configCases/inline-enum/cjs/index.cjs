const enums = require("../basic/enum");

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should not inline enums for cjs", () => {
  // START:A
  expect(enums.E.A).toBe(0);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`(enums.E.A).toBe(0)`)).toBe(true);
})

it("should keep the module for cjs", () => {
  const noInlinedModuleIds = ["../basic/enum.ts"];
  noInlinedModuleIds.forEach(m => {
    expect(generated.includes(`"${m}": (function`)).toBe(true);
  })
})
