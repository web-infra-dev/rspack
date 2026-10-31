import './style.css';

const fs = require('fs');
const path = require('path');

it('should expand custom media and remove its definitions', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	expect(css).not.toContain('@custom-media');
	expect(css).not.toContain('(--wide)');
	expect(css).toContain('@media (width >= 800px)');
	expect(css).toContain('.example');
});
