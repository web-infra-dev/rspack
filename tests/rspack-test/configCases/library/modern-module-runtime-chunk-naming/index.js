const fs = require('fs');

it('names the split runtime chunk after its entry to avoid cross-lib collisions', () => {
	const files = fs.readdirSync(__dirname);

	// Each entry's runtime chunk is named after the entry, so two libs that share
	// this output directory cannot both emit the same `<id>.js` and clobber each
	// other (see https://github.com/web-infra-dev/rspack/pull/14508).
	expect(files).toContain('a_runtime.js');
	expect(files).toContain('b_runtime.js');

	// no bare id-named chunk remains that two libs could collide on
	expect(files.some((f) => /^\d+\.js$/.test(f))).toBe(false);
});
