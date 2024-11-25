it("should inject trigger runtime module after normal runtime module", async function () {
  expect(__webpack_require__.mockNormal).toBe("normal");
  expect(__webpack_require__.mockTrigger).toBe("trigger");
  const fs = require("fs");
  const content = fs.readFileSync(__filename, 'utf-8');
  const triggerIndex = content.indexOf(`__webpack_require__.mockTrigger = "trigger"`);
  const normalIndex = content.indexOf(`__webpack_require__.mockNormal = "normal"`);
  expect(normalIndex).toBeLessThan(triggerIndex);
});