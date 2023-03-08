const path = require('path');
module.exports = (env,argv) =>  {
  console.log('env:',env,argv)
  return {
    context: __dirname,
    mode: 'development',
    entry: {
      main: './index.js'
    },
    output: {
      path: path.resolve(__dirname, 'dist')
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap("DEMO", (compilation) => {
            compilation.hooks.resolve.tapAsync("DEMO", (args, callback) => {
              console.log(args)
              if (args.specifier === 'rspack:a') {
                callback(null, { path: path.resolve(__dirname, './rspack-a.js') })
              } else {
                callback(null, undefined)
              }
            })
          })
        }
      }
    ]
  }
}