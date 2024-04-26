module.exports = {
  description: "Testing proxy methods on errors: test shift&unshift",
  options() {
    return {
      entry: "./resolve-fail-esm",
      plugins: [
        compiler => {
          compiler.hooks.afterCompile.tap(
            "test shift and unshift",
            compilation => {
              compilation.errors.shift();
              compilation.errors.unshift("test unshift");
            }
          );
        }
      ]
    };
  },
  async check(diagnostics) {
    expect(diagnostics).toMatchInlineSnapshot(`
      Object {
        "errors": Array [
          Object {
            "formatted": "  × test unshift\\n",
            "message": "  × test unshift\\n",
          },
        ],
        "warnings": Array [],
      }
    `);
  }
};
