/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  builtins: {
    html: [{
      template: './index.html'
    }]
  },
  context: __dirname,
  mode: "development",
  entry: {
    main: ["./index.js"],
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  infrastructureLogging: {
    debug: false
  }
};
