it("should not contain full hash runtime module", async () => {
  await import("./index.css");

  const chunk = __non_webpack_require__("fs").readFileSync(__filename, "utf-8");
  const hashRuntime = ["__webpack_require__", "h"].join(".") // use join() here to avoid compile time evaluation
  expect(chunk).not.toContain(hashRuntime);
});
