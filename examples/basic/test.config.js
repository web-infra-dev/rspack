/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  builtins: {
    html: [{
      template: './index.html'
    }],
    minify: false
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
