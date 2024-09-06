it('should inject bundlerInfo when library is falsy', () => {
    const content = require('fs').readFileSync(
		require('path').resolve(__dirname, "bundle0.js"),
		"utf-8"
	);
    expect(content).toContain('__webpack_require__.rv')
})