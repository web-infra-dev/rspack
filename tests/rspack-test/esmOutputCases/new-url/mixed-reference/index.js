import assetPath from './asset.txt'

const assetUrl = new URL('./asset.txt', import.meta.url)

// Referencing the same asset through both an `import` and a `new URL()` must not
// pull the asset module out of scope hoisting, which would force it into the
// `__webpack_require__` module registry and drag the runtime into the output.
it('should reference the same asset through both `import` and `new URL`', () => {
	expect(assetPath.endsWith('asset.txt')).toBe(true)
	expect(assetUrl.href.endsWith('asset.txt')).toBe(true)
})
