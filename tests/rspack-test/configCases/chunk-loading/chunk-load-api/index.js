it("should have __webpack_chunk_load__ API available", () => {
    expect(__webpack_chunk_load__).toBeDefined();
    expect(typeof __webpack_chunk_load__).toBe("function");
});
