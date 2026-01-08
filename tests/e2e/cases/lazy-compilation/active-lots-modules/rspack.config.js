const { rspack } = require('@rspack/core');

let lotsLongFileNameVirtualFiles = {};
let longStr = new Array(1024).fill('a').join('');
for (let i = 0; i < 20; i++) {
  lotsLongFileNameVirtualFiles[
    `src/virtual_with_a_very_long_file_name_number_${longStr}_${i}.js`
  ] = `"dynamic_imported"`;
}
let allFiles = Object.keys(lotsLongFileNameVirtualFiles);
lotsLongFileNameVirtualFiles['src/virtual_index.js'] = `
  Promise.all([
    ${allFiles.map((file) => `import('./${file.slice(3)}')\n`).join(',')}
  ]).then(()=> document.body.innerHTML = 'All Modules Loaded');
`;

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/virtual_index.js',
  mode: 'development',
  lazyCompilation: true,
  devServer: {
    hot: true,
    port: 5678,
  },
  plugins: [
    new rspack.HtmlRspackPlugin(),
    new rspack.experiments.VirtualModulesPlugin(lotsLongFileNameVirtualFiles),
  ],
  experiments: {
    useInputFileSystem: [/virtual/],
  },
};
