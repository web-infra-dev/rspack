const { execFileSync } = require('node:child_process')
const fs = require('node:fs')
const path = require('node:path')

module.exports = {
  snapshotFileFilter(file) {
    return file === 'main.mjs'
  },
  snapshotContent(content) {
    return content
      .split('\n')
      .filter(line =>
        /^(?:import .*?(?:\.\/runtime\.mjs|\.\/assets\/(?:__webpack_require__|rspackRequire)\.mjs)|module\.exports = |export )/.test(
          line,
        ),
      )
      .join('\n')
  },
  afterExecute(options) {
    const outputFile = path.join(options.output.path, 'main.mjs')
    const source = fs.readFileSync(outputFile, 'utf8')

    execFileSync(process.execPath, ['--check', outputFile])

    const assetBindings = new Map(
      Array.from(
        source.matchAll(
          /^import ([A-Za-z_$][\w$]*) from "\.\/assets\/((?:__webpack_require__|rspackRequire)\.mjs)";$/gm,
        ),
        match => [match[2], match[1]],
      ),
    )

    expect(assetBindings.size).toBe(2)
    expect(new Set(assetBindings.values()).size).toBe(assetBindings.size)

    const runtimeBindings = new Set(
      source
        .match(/^import \{\s*([^}]+)\s*\} from "\.\/runtime\.mjs";$/m)?.[1]
        .split(',')
        .map(specifier => specifier.trim().split(/\s+as\s+/).at(-1)) || [],
    )
    let conflictingRuntimeBindings = 0
    for (const [asset, binding] of assetBindings) {
      const expectedRuntimeBinding = asset.slice(0, -'.mjs'.length)
      if (runtimeBindings.has(expectedRuntimeBinding)) {
        conflictingRuntimeBindings += 1
        expect(binding).not.toBe(expectedRuntimeBinding)
      }
      expect(source).toContain(`module.exports = ${binding};`)
    }
    expect(conflictingRuntimeBindings).toBe(1)
  },
}
