it("should inject trigger runtime module after normal runtime module", async function () {
  expect(__webpack_require__.mockNormal).toBe("normal");
  expect(__webpack_require__.mockTrigger).toBe("trigger");
  const fs = require("fs");
  const content = fs.readFileSync(__filename, 'utf-8');
  const scope = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
    ? "__rspack_context" : "__webpack_require__";
  const triggerIndex = content.indexOf(`${scope}.mockTrigger = "trigger"`);
  const normalIndex = content.indexOf(`${scope}.mockNormal = "normal"`);
  expect(normalIndex).toBeGreaterThanOrEqual(0);
  expect(normalIndex).toBeLessThan(triggerIndex);
});
