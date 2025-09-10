it('should ensure max size fit', () => {
	const names = new Array(4).fill('').map((_, i) => {
		return `50k-${i + 1}.js`;
	});
	names.forEach((name) => {
		require('./aaa/' + name)
		require('./bbb/' + name)
		require('./ccc/' + name)
	})
})
