import a from "./module"

it("should have correct entrypoints", function() {
  expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0", "bundle1"]);
  expect(a).toBe(1)
})
