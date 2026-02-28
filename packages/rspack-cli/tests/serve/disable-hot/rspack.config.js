module.exports = {
  mode: 'development',
  devServer: {
    hot: false,
    onListening(devServer) {
      const { hot } = devServer.options;
      console.log(JSON.stringify({ hot }));
    },
  },
};
