import fs from "node:fs";
import path from "node:path";

export default {
  snapshotFileFilter(file) {
    return file === 'main.mjs'
  },
  snapshotContent(content) {
    return content
      .split('\n')
      .filter(line => /^(?:import value_asset |module\.exports = value_asset;)/.test(line))
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
    expect(source).not.toMatch(/\brequire\(/)
  },
}
