import { parse } from "acorn";
import fs from "node:fs";
import path from "node:path";

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

export default {
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

    expectUniqueImportBindings(source)
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
