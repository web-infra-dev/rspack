module.exports = {
  description: "Testing proxy methods on warnings: test pop",
  options() {
    return {
      entry: "./require.main.require",
      plugins: [
        compiler => {
          compiler.hooks.afterCompile.tap("test pop", compilation => {
            compilation.warnings.pop();
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
