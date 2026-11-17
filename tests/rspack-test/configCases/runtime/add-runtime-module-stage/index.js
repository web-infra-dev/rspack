it("should inject trigger runtime module after normal runtime module", async function () {
  expect(__webpack_require__.mockNormal).toBe("normal");
  expect(__webpack_require__.mockTrigger).toBe("trigger");
  expect(__webpack_require__.stageOrder).toEqual(["normal", "trigger"]);
});
