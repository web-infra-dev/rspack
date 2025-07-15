module.exports = {
  entry: { main: "./src/Three.js" },
  mode: "production", 
  devtool: false,
  optimization: {
    minimize: false,
    usedExports: true,
    providedExports: true,
    sideEffects: false,
    concatenateModules: true,
    innerGraph: true,
    removeAvailableModules: true,
    removeEmptyChunks: true,
    mergeDuplicateChunks: true
  }
};
