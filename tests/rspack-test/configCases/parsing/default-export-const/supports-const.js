import d from "./supports-const.js"

try {
  d
} catch (e) {
  it('should have TDZ error', () => {
    expect(e.message).toBe("Cannot access '__WEBPACK_DEFAULT_EXPORT__' before initialization");
  })
}

export default 1;
