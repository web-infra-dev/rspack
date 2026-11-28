it("should preserve external request names starting with by", () => {
  expect(require("bytes")).toBe(44);
  expect(require("./app")).toBe(42);
  expect(require("./other")).toEqual([42, 45]);
});
