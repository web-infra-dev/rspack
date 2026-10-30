export default {
  resolve: {
    byDependency: { less: { conditionNames: ['less', 'style', 'custom'] } },
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          './assert-map-loader.mjs',
          { loader: 'builtin:less-loader', options: { sourceMap: true } },
        ],
      },
    ],
  },
};
