it("should have correct result", () => {
  import("./a").then(() => {
    if (WATCH_STEP === '1') {
      expect(global.__test_value__).toBe("main");
      expect(__webpack_require__.j).toBe("main");
    } else {
      expect(global.__test_value__).toBe(42);
      expect(__webpack_require__.j).toBe(undefined);
    }
  });
})
