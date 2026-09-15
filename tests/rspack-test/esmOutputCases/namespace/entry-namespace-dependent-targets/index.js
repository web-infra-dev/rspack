export * as first from "./first";
export * as shared from "./shared";
export * as last from "./last";
import { trace } from "./trace";

it("preserves order after excluding dependent namespace targets", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.dependent.mjs");
  expect(mod.first.value).toBe("shared:first");
  expect(mod.shared.value).toBe("shared");
  expect(mod.last.value).toBe("last");
  expect(trace).toEqual(["shared", "first", "last"]);
});
