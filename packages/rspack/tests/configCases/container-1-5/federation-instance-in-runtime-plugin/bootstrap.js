export function test(it) {
  it("should able to access federation object on __webpack_require__", () => {
    expect(__webpack_require__.federation.test).toBe(1)
  })
}
