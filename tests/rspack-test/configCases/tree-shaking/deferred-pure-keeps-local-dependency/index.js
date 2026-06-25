import { c } from "./re-export";

it("should keep local bindings read by retained deferred-impure expressions", () => {
  expect(c).toBeUndefined();
});
