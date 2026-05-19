it('should not force inject bundlerInfo when library is falsy', () => {
  const content = require("fs").readFileSync(
    require("path").resolve(__dirname, "bundle0.js"),
    "utf-8"
  );

  expect(content).not.toMatch(/(^|[^"'`])__webpack_require__\.rv =/m);
  expect(content).not.toMatch(/(^|[^"'`])__webpack_require__\.ruid =/m);
});
