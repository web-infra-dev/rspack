/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  context: __dirname,
  mode: 'development',
  builtins: {
    html: [{
      template: './index.html'
    }],
    treeShaking: true,
    sideEffects: true
  },
  target: ['web', 'es5'],
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
