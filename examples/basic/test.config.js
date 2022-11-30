const path = require('path');
/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  builtins: {
    html: [{
      template: './index.html'
    }],
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
  },
  output: {
    path: path.resolve(__dirname, 'dist')
  }
};
