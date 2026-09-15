const fs = require('node:fs')
const path = require('node:path')

let outputPath
let initialAssetSymbol

module.exports = {
  findBundle(_index, options) {
    outputPath = options.output.path
    return []
  },
  checkStats(step) {
    const source = fs.readFileSync(path.join(outputPath, 'main.mjs'), 'utf8')
    const imports = Array.from(
      source.matchAll(
        /^import ([A-Za-z_$][\w$]*) from "\.\/assets\/([^"/]+\.asset\.mjs)";$/gm,
      ),
      match => ({ symbol: match[1], asset: match[2] }),
    )
    const symbolsByAsset = new Map(
      imports.map(({ asset, symbol }) => [asset, symbol]),
    )

    expect(imports).toHaveLength(step === '0' ? 1 : 2)
    expect(new Set(imports.map(({ symbol }) => symbol)).size).toBe(
      imports.length,
    )
    expect(source).not.toMatch(/\brequire\(/)

    for (const { symbol } of imports) {
      expect(source).toContain(`module.exports = ${symbol};`)
    }

    if (step === '0') {
      initialAssetSymbol = symbolsByAsset.get('a.asset.mjs')
      expect(initialAssetSymbol).toBeDefined()
    } else {
      expect(symbolsByAsset.get('a.asset.mjs')).toBe(initialAssetSymbol)
      expect(symbolsByAsset.get('b.asset.mjs')).toBeDefined()
    }

    return true
  },
}
