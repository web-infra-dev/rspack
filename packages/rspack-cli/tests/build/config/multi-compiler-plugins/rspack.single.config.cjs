const path = require('node:path');

module.exports = {
  name: 'single',
  mode: 'development',
  entry: './a.js',
  output: { path: path.resolve(__dirname, 'dist') },
  plugins: [
    (compiler) => {
      console.log(`child plugin: compilers ${compiler.compilers?.length}`);
    },
  ],
};
