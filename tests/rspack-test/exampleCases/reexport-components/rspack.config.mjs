import path from 'node:path';
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        include: [path.resolve(import.meta.dirname, '.')],
        use: {
          loader: 'babel-loader',
          options: {
            presets: ['@babel/react'],
          },
        },
      },
    ],
  },
  optimization: {
    // get readable names in production too
    chunkIds: 'named',
    moduleIds: 'named',
    // enable some optimizations in dev mode too for showcasing
    sideEffects: true,
    usedExports: true,
  },
};
