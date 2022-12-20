/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  mode: 'development',
  context: __dirname,
  builtins: {
    html: [{
      template: './index.html'
    }],
    treeShaking: false,
    sideEffects: false
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
  target: ['web', 'es5']
};
