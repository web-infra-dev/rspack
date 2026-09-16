import path from 'node:path';
export default {
  // mode: "development" || "production",
  entry: {
    alpha: './alpha',
    beta: './beta',
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: 'MyLibrary.[name].js',
    library: { type: 'umd', name: ['MyLibrary', '[name]'] },
  },
};
