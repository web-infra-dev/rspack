const mfScope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;

export function test(it) {
  it("should able to access federation object on __webpack_require__", () => {
    expect(mfScope.federation.test).toBe(1)
    expect(mfScope.federation.rootOutputDir).toBe("../")
  })
}
