it("should inject trigger runtime module after normal runtime module", async function () {
  expect(__webpack_require__.mockNormal).toBe("normal");
  expect(__webpack_require__.mockTrigger).toBe("trigger");
  const fs = require("fs");
  const content = fs.readFileSync(__filename, 'utf-8');
  const statements = [
    {
      normal: `__webpack_require__.mockNormal = "normal";`,
      trigger: `__webpack_require__.mockTrigger = "trigger";`
    },
    {
      normal: `__rspack_context.mockNormal = "normal";`,
      trigger: `__rspack_context.mockTrigger = "trigger";`
    }
  ];
  const statement = statements.find(({ normal, trigger }) => {
    return content.includes(normal) && content.includes(trigger);
  });
  expect(statement).toBeTruthy();
  const triggerIndex = content.indexOf(statement.trigger);
  const normalIndex = content.indexOf(statement.normal);
  expect(normalIndex).toBeLessThan(triggerIndex);
});
