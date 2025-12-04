import d from "./supports-const.js"

try {
  d
} catch (e) {
  it('should have TDZ error', () => {
    expect(e.message).toBe("Cannot access '__rspack_default_export' before initialization");
  })
}

export default 1;
