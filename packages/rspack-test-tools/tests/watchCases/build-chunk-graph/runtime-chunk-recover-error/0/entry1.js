it("should have correct value", async () => {
  try {
    const { value } = await import("./lib1");
    switch (WATCH_STEP) {
      case "0":
        expect(value).toBe("ok");
        break;
      case "2":
        expect(value).toBe("fixed");
        break;
      default:
        throw new Error(`Unknown WATCH_STEP: ${WATCH_STEP}`);
    }
  } catch (e) {
    expect(WATCH_STEP).toBe("1");
    expect(e.message).toMatch(/JavaScript parse error/);
  }
});
