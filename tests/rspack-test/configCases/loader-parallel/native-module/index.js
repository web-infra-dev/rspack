it("uses the task's native module inside the worker facade", () => {
  expect(require("./value.js")).toBe("native module");
});
