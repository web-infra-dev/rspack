it('should add shims for js/esm modules', () => {
	const fs = __non_webpack_require__('fs');
	const path = __non_webpack_require__('path');
	const content = fs.readFileSync(path.resolve(__dirname, 'bundle.mjs'), 'utf-8');
	expect(content).toContain('__webpack_fileURLToPath__');
})
