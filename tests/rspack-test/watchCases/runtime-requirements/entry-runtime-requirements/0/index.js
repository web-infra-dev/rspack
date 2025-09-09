it("should have correct result", () => {
  import("./a").then(() => {
    if (WATCH_STEP === '1') {
      expect(global.value).toBe("main");
      expect(__webpack_require__.j).toBe("main");
    } else {
      expect(global.value).toBe(42);
      expect(__webpack_require__.j).toBe(undefined);
    }
  });
})
