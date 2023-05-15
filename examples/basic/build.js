const { rspack } = require('../../packages/rspack')
const config = require('./rspack.config')

// call rspack
const compiler = rspack(config)
// compiler.run
compiler.run((err, stats) => {
  let  statsString = stats.toString();
	console.log('statsString', statsString)
  if (err) {
    console.log(err)
  }
})
