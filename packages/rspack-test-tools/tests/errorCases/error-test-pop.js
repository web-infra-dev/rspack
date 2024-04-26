module.exports = {
  description: "Testing proxy methods on errors: test pop",
  options() {
    return {
      entry: "./resolve-fail-esm",
      plugins: [
        compiler => {
          compiler.hooks.afterCompile.tap("test pop", compilation => {
            compilation.errors.pop();
          });
        }
      ]
    };
  },
  async check(diagnostics) {
    expect(diagnostics).toMatchInlineSnapshot(`
      Object {
        "errors": Array [],
        "warnings": Array [],
      }
    `);
  }
};
