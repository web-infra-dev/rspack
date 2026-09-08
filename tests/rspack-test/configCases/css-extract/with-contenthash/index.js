it("should not contain full hash runtime module", async () => {
  await import("./index.css");

  const chunk = require("fs").readFileSync(__filename, "utf-8");
  // Match the full hash accessor itself, not any longer property name that happens to
  // start with the same letter (e.g. the extract-css HMR filename map) - a plain substring
  // check would false-positive on those too.
  const hashRuntime = new RegExp(["__webpack_require__", "h"].join("\\.") + "\\b") // use join() here to avoid compile time evaluation
  expect(chunk).not.toMatch(hashRuntime);
});
