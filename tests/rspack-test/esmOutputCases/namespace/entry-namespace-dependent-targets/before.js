import "./last";
export * as first from "./first";
import { trace } from "./trace";

it("does not split a namespace after an earlier side-effect dependency", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.before.mjs");
  expect(mod.first.value).toBe("shared:first");
  expect(trace).toEqual(["last", "shared", "first"]);
});
