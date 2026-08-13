const { parse } = require('acorn')
const fs = require('node:fs')
const path = require('node:path')

function expectUniqueImportBindings(source) {
  const program = parse(source, {
    ecmaVersion: 'latest',
    sourceType: 'module',
  })
  const importBindings = program.body
    .filter(statement => statement.type === 'ImportDeclaration')
    .flatMap(statement => statement.specifiers.map(specifier => specifier.local.name))

  expect(new Set(importBindings).size).toBe(importBindings.length)
}

module.exports = {
  snapshotFileFilter(file) {
    return file === 'main.mjs'
  },
  snapshotContent(content) {
    return content
      .split('\n')
      .filter(line =>
        /^(?:import same_name_asset|module\.exports = same_name_asset|const (?:index_)?same_name_asset|export )/.test(
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
    expectUniqueImportBindings(source)
    const importedSymbols = Array.from(
      source.matchAll(
        /^import ([A-Za-z_$][\w$]*) from "\.\/assets\/same[-_]name\.asset\.mjs";$/gm,
      ),
      match => match[1],
    )

    expect(importedSymbols).toEqual(['same_name_asset', 'same_name_asset_0'])
    expect(new Set(importedSymbols).size).toBe(importedSymbols.length)
    expect(source).toContain('module.exports = same_name_asset;')
    expect(source).toContain('module.exports = same_name_asset_0;')
    expect(source).toContain(
      "const index_same_name_asset = 'first application value'",
    )
    expect(source).toContain(
      "const index_same_name_asset_0 = 'second application value'",
    )
    expect(source).toContain(
      'same_name_asset: index_same_name_asset,',
    )
    expect(source).toContain(
      'same_name_asset_0: index_same_name_asset_0,',
    )
    expect(source).not.toMatch(/\brequire\(/)
  },
}
