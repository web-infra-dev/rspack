it('should pass additional data to next loader without execute toJSON', () => {
	expect(require('./a.js')).toStrictEqual({
		str: 'str',
		num: 1
	})
})
