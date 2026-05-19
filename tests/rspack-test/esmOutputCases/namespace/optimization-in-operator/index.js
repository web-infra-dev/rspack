import * as foo from "./foo.js";

export const missing = "d" in foo ? foo.d() : "nope";
export const present = "c" in foo ? foo.c() : "nope";
export const hasC = "c" in foo;

it("should optimize namespace existence checks with the in operator", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.missing).toBe("nope");
  expect(mod.present).toBe("c");
  expect(mod.hasC).toBe(true);
});
