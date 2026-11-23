const fs = require('fs');
const path = require('path');

it('should render the shared modules', async () => {
	const { App, value } = await import('./module.js');
	expect(App()).toBe('XReact YReact');
	expect(value).toBe('shared');
});

for (const suffix of ['', '-stats']) {
	const file = `mf-${CASE_INDEX}${suffix}.json`;
	it(`should preserve shared names, versions, ranges and assets in ${file}`, () => {
		const { shared } = JSON.parse(
			fs.readFileSync(path.join(__dirname, file), 'utf-8'),
		);
		expect(shared).toHaveLength(3);
		const requiredVersions = Object.fromEntries(
			shared.map(item => [item.name, item.requiredVersion]),
		);

		expect(requiredVersions).toEqual({
			xreact: '^1.0.0 || ^2.0.0',
			yreact: '^2.0.0 || ^3.0.0',
			[SHARE_KEY]: null,
		});
		const versions = {
			xreact: '2.1.0',
			yreact: '2.1.0',
			[SHARE_KEY]: SHARED_VERSION,
		};
		for (const item of shared) {
			expect(item.id).toBe(`container${CASE_INDEX}:${item.name}`);
			expect(item.version).toBe(versions[item.name]);
			expect(item.assets.js.sync.length).toBeGreaterThan(0);
			if (suffix === '-stats' && (item.name !== SHARE_KEY || CONSUME_SHARED)) {
				expect(item.usedIn).toContain('module.js');
			}
		}
	});
}
