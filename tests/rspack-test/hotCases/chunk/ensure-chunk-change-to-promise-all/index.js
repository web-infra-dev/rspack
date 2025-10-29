import { test } from "./chunk"

it("should still works when ensure chunk causes the parent chunk change", async () => {
  const react = await test(0);
  expect(react).toBe(42);
  import.meta.webpackHot.accept("./chunk");
  await NEXT_HMR();
  const vue = await test(1);
  expect(vue).toBe(43);
});
