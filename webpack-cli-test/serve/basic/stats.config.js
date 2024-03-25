module.exports = {
  mode: "development",
  devtool: false,
  devServer: {
    devMiddleware: {
      stats: "minimal",
    },
    client: {
      logging: "info",
    },
  },
};
