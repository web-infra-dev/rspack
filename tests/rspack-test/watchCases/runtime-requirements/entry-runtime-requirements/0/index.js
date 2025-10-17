it("should have correct result", () => {
  import("./a").then(() => {
    if (WATCH_STEP === '1') {
      expect(temp.__test_value__).toBe("main");
      expect(__webpack_require__.j).toBe("main");
    } else {
      expect(temp.__test_value__).toBe(42);
      expect(__webpack_require__.j).toBe(undefined);
    }
  });
})
