import { defineConfig } from '../../../dist/index.js';

export default defineConfig({
  mode: 'development',
  devServer: {
    hot: false,
    onListening(devServer) {
      const { hot } = devServer.options;
      console.log(JSON.stringify({ hot }));
    },
  },
});
