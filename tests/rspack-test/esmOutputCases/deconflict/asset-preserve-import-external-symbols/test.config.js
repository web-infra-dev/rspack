const { parse } = require('acorn')
const { execFileSync } = require('node:child_process')
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
  return importBindings
}

module.exports = {
  snapshotFileFilter(file) {
    return file === 'main.mjs'
  },
  snapshotContent(content) {
    return content
      .split('\n')
      .filter(line =>
        /^(?:import .*?(?:node:url|"fs"|\.\/assets\/)|var __rspack_import_meta_dirname__|module\.exports = |const (?:index_)?generatedDirname|export )/.test(
          line,
        ),
      )
      .join('\n')
  },
  afterExecute(options) {
    const outputFile = path.join(options.output.path, 'main.mjs')
    const source = fs.readFileSync(outputFile, 'utf8')

    execFileSync(process.execPath, ['--check', outputFile])
    const allImportBindings = expectUniqueImportBindings(source)

    const readFileImport = source.match(
      /^import \{\s*readFile(?: as ([A-Za-z_$][\w$]*))?\s*\} from "fs";$/m,
    )
    const fileURLToPathBindings = Array.from(
      source.matchAll(
        /^import \{\s*fileURLToPath(?: as ([A-Za-z_$][\w$]*))?\s*\} from "node:url";$/gm,
      ),
      match => match[1] || 'fileURLToPath',
    )
    const helperBinding = fileURLToPathBindings.find(
      binding => binding === '__rspack_fileURLToPath',
    )
    const externalBindings = {
      readFile: readFileImport?.[1] || (readFileImport ? 'readFile' : undefined),
      fileURLToPath: fileURLToPathBindings.find(
        binding => binding !== helperBinding,
      ),
    }
    const assetBindings = new Map(
      Array.from(
        source.matchAll(
          /^import ([A-Za-z_$][\w$]*) from "\.\/assets\/((?:fileURLToPath|readFile)\.mjs)";$/gm,
        ),
        match => [match[2], match[1]],
      ),
    )

    expect(helperBinding).toBe('__rspack_fileURLToPath')
    expect(fileURLToPathBindings).toHaveLength(2)
    expect(externalBindings.readFile).toBeDefined()
    expect(externalBindings.fileURLToPath).toBeDefined()
    expect(assetBindings.size).toBe(2)

    expect(allImportBindings).toEqual(
      expect.arrayContaining([
        helperBinding,
        ...Object.values(externalBindings),
        ...assetBindings.values(),
      ]),
    )

    expect(assetBindings.get('readFile.mjs')).not.toBe(
      externalBindings.readFile,
    )
    expect(assetBindings.get('fileURLToPath.mjs')).not.toBe(
      externalBindings.fileURLToPath,
    )
    for (const binding of assetBindings.values()) {
      expect(source).toContain(`module.exports = ${binding};`)
    }
  },
}
