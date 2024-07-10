/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index",
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterCompile.tap("TestPlugin", compilation => {
          const errorHide = new Error("push message hide");
          errorHide.hideStack = true;
          errorHide.stack = "push stack hide";
          compilation.errors.push(errorHide);

          const error = new Error("push message");
          error.stack = "push stack";
          compilation.errors.push(error);
        });
        compiler.hooks.done.tap("TestPlugin", stats => {
          const errors = stats.toJson({ errors: true }).errors;
          for (let error of errors) {
            if (error.message.includes("hide")) {
              expect(typeof error.details).toBe("string");
              expect(error.message.includes("stack")).toBeFalsy;
            } else {
              expect(typeof error.details).toBe("undefined");
            }
            expect(typeof error.stack).toBe("string");
          }
        });
      }
    }
  ]
};
