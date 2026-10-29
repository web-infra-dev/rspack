import { defineConfig } from '../../../dist/index.js';

export default defineConfig({
  mode: 'development',
  devtool: false,
  devServer: {
    hot: false,
    port: 8080,
    host: '127.0.0.1',
    onListening(devServer) {
      const { hot, host, port } = devServer.options;
      console.log(JSON.stringify({ hot, host, port }));
    },
  },
});
