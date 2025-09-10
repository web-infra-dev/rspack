import * as namespace from './export-imported'

// consume
namespace;

it('should have correct consistent order', () => {
	const fs = __non_webpack_require__('fs')
	const path = __non_webpack_require__('path')
	/** @type {string} */
	const code = fs.readFileSync(path.resolve(__dirname, './bundle.js')).toString()

	const re = /;\/\/ CONCATENATED MODULE: (.*)\n/g;

	const [a, b, c, d, e, f, g] = [...code.matchAll(re)]
	expect(a[1]).toBe('./a.js')
	expect(b[1]).toBe('./b.js')
	expect(c[1]).toBe('./c.js')
	expect(d[1]).toBe('./d.js')
	expect(e[1]).toBe('./e.js')
	expect(f[1]).toBe('./f.js')
	expect(g[1]).toBe('./g.js')
})
