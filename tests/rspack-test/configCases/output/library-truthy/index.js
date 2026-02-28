it('should not inject bundlerInfo when library is truthy', () => {
    const content = require('fs').readFileSync(
		require('path').resolve(__dirname, "bundle0.mjs"),
		"utf-8"
	);
    expect(content).not.toContain('__webpack_require__' + '.rv')
})