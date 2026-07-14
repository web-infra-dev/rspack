const { lazyCompilationMiddleware } = require("@rspack/core");
const path = require("path");

const context = path.join(__dirname, "../fixtures");
const middlewareKey = "lazyCompilationMiddleware";

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
module.exports = {
  description: "lazy compilation invalidates only its owning compiler",
  options() {
    return [
      {
        entry: "./esm/a.js",
        context,
        lazyCompilation: false,
      },
      {
        entry: "./esm/b.js",
        context,
        lazyCompilation: {
          entries: true,
        },
      },
    ];
  },
  compiler(context, compiler) {
    context.setValue(middlewareKey, lazyCompilationMiddleware(compiler));
  },
  async build(context, compiler) {
    let multiCompilerInvalidations = 0;
    let lazyCompilerInvalidations = 0;

    compiler.watching = {
      invalidate: () => {
        multiCompilerInvalidations += 1;
      },
      close: callback => callback(),
    };
    compiler.compilers[1].watching = {
      invalidate: () => {
        lazyCompilerInvalidations += 1;
      },
      close: callback => {
        compiler.compilers[1].watching = undefined;
        callback();
      },
    };

    const middleware = context.getValue(middlewareKey);
    await middleware(
      {
        method: "POST",
        url: "/_rspack/lazy/trigger__0",
        body: ["lazy-entry"],
      },
      {
        writeHead() {},
        write() {},
        end() {},
      },
      () => {},
    );

    expect(lazyCompilerInvalidations).toBe(1);
    expect(multiCompilerInvalidations).toBe(0);
  },
};
