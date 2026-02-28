const styles = require('./entry').default
require('./entry2')

it("css hmr", async () => {
	expect(styles).toHaveProperty('foo')
	await NEXT_HMR();
	const updatedStyles = require('./entry').default
	expect(updatedStyles).toHaveProperty('bar')
});

module.hot.accept('./entry');