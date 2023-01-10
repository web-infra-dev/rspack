/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  context: __dirname,
  builtins: {
    html: [{
      template: './index.html'
    }],
    treeShaking: true,
    sideEffects: true
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
