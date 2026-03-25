import * as fsNs from "fs";

const unused = fsNs.readFile;

export default "main";

it("should not count external namespace property access as a side effect", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(unused).toBeDefined();
  expect(mod.default).toBe("main");
});
