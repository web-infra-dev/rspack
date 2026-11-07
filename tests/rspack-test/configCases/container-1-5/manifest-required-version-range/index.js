const fs = require('fs');
const path = require('path');

it('should render the shared modules', async () => {
	const { App } = await import('./module.js');
	expect(App()).toBe('XReact YReact');
});

for (const file of ['mf-manifest.json', 'mf-stats.json']) {
	it(`should keep whole requiredVersion ranges in ${file}`, () => {
		const { shared } = JSON.parse(fs.readFileSync(path.join(__dirname, file), 'utf-8'));
		const requiredVersions = Object.fromEntries(
			shared.map(item => [item.name, item.requiredVersion]),
		);

		expect(requiredVersions).toEqual({
			xreact: '^1.0.0 || ^2.0.0',
			yreact: '^2.0.0 || ^3.0.0',
		});
	});
}
