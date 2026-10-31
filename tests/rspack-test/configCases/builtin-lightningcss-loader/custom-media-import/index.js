import './index.css';

const fs = require('fs');
const path = require('path');

it('should keep custom media definitions referenced by @import conditions', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	expect(css).toContain('@custom-media --wide (width >= 800px);');
	expect(css).toContain('.imported');
	expect(css).toContain('.local');
});
