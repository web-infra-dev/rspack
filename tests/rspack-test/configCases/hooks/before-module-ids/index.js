import a from "./a";

it("should assign custom module ids via beforeModuleIds hook", () => {
  expect(a).toBe("a");
});
