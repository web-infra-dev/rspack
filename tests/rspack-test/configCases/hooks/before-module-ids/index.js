import a from "./a";
import b from "./b";

it("should assign custom module ids via beforeModuleIds hook", () => {
  expect(a).toBe("a");
  expect(b).toBe("b");
});
