/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  mode: 'development',
  context: __dirname,
  builtins: {
    // html: [{
    //   template: './index.html'
    // }],
    treeShaking: true,
    sideEffects: true
  },
  module: {
    rules: [
      {
        test: {
          type: "regexp",
          matcher: "a.js$",
        },
        sideEffects: false
      }
    ],
  },
  context: __dirname,
  entry: {
    main: {
      import: ["./index.js"],
    }
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  infrastructureLogging: {
    debug: false
  }
};
