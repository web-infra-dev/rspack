/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: false,
  externals: [
    {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    function ({ request, dependencyType }, callback) {
      if (/^(\/\/|custom?:\/\/)/.test(request)) {
        if (dependencyType === 'css-import')
          return callback(null, request, 'css-import');
        if (dependencyType === 'url') return callback(null, request, 'css-url');
        return callback(null, `var '${request}'`);
      }
      return callback();
    },
  ],
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
