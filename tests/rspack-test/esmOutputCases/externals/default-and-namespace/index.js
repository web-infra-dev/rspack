import fsDefault from "fs";
import * as fsNs from "fs";

export default fsDefault;
export { fsNs };

it("should distinguish external default and namespace imports", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toBe(fsDefault);
  expect(mod.default).toBe(fsNs.default);
  expect(typeof mod.fsNs.readFile).toBe("function");
});
