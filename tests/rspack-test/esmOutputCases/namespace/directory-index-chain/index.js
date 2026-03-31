import * as things from "./folders";

it("should resolve a namespace import through nested index re-export chains", () => {
  expect(typeof things.foo).toBe("function");
  expect(things.foo()).toBe("hi there");
});
