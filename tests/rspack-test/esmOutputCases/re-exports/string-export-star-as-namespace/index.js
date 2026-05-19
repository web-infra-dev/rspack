export { "some import" as "some export" } from "./foo";
export * as "all the stuff" from "./foo";

it("should support string export names on namespace re-exports", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod["some export"]).toBe(42);
  expect(mod["all the stuff"]["some import"]).toBe(42);
  expect(mod["all the stuff"].named).toBe("hello");
});
