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
        /^(?:import value_asset |module\.exports = value_asset;|const (?:index_)?value_asset|const shorthand|export )/.test(
          line,
        ),
      )
      .join('\n')
  },
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, 'main.mjs'),
      'utf8',
    )

    expect(source).toContain(
      'import value_asset from "./assets/value.asset.mjs";',
    )
    expect(source).toContain('module.exports = value_asset;')
    expect(source).toContain(
      "const index_value_asset = 'application value'",
    )
    expect(source).toContain(
      'const shorthand = { value_asset: index_value_asset }',
    )
    expect(source).not.toMatch(/\brequire\(/)
  },
}
