import big from "./big.svg";

export const observedTopLevelThis = this;

it("keeps the generated bundle executable", async () => {
  expect(big).toMatch(/^data:image\/svg\+xml/);
  expect(observedTopLevelThis).toBeUndefined();
  expect((await import("./a.js")).default).toBe("a");
});
