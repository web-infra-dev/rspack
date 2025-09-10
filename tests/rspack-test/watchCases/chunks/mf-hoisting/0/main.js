import getWorker from "./getWorker";

it("should have correct value", () => {
  expect(getWorker().testName).toBe("test worker 0");
})
