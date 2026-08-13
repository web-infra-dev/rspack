const fs = require('node:fs')
const path = require('node:path')

let outputPath
let initialMainFilename

module.exports = {
  findBundle(_index, options) {
    outputPath = options.output.path
    return []
  },
  checkStats(step, stats) {
    const mainAsset = stats.assets.find(
      asset => asset.name.startsWith('main.') && asset.name.endsWith('.mjs'),
    )
    expect(mainAsset).toBeDefined()

    const source = fs.readFileSync(
      path.join(outputPath, mainAsset.name),
      'utf8',
    )
    const assetDirectory = step === '0' ? 'assets' : 'renamed'
    expect(source).toMatch(
      new RegExp(
        `^import [A-Za-z_$][\\w$]* from "\\./${assetDirectory}/value\\.asset\\.mjs";$`,
        'm',
      ),
    )

    if (step === '0') {
      initialMainFilename = mainAsset.name
    } else {
      expect(mainAsset.name).not.toBe(initialMainFilename)
    }

    return true
  },
}
