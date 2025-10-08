import getWorker from "./getWorker";

it("should rebuild successfully", () => {
  expect(getWorker().testName).toBe("test worker 1");
})
