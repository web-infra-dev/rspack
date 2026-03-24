import * as fsNs from "fs";

it("should allow probing external namespace properties", () => {
  const presentKey = ["read", "File"].join("");
  const missingKey = ["not", "There"].join("");

  expect(fsNs.default).toBeDefined();
  expect(Reflect.get(fsNs, presentKey)).toBeDefined();
  expect(Reflect.get(fsNs, missingKey)).toBeUndefined();
});
