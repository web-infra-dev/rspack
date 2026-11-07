const path = require('node:path');

const output = { path: path.resolve(__dirname, 'dist') };

module.exports = Object.assign(
  [
    { name: 'a', mode: 'development', entry: './a.js', output },
    { name: 'b', mode: 'development', entry: './b.js', output },
  ],
  {
    parallelism: 1,
    plugins: [
      (compiler) => {
        console.log(
          `multi plugin: ${compiler.compilers.length} compilers, parallelism ${compiler.options.parallelism}`,
        );
      },
    ],
  },
);
