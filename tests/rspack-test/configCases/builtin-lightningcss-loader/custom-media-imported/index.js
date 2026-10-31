import './component.css';
import './override.css';

const fs = require('fs');
const path = require('path');

it('should expand custom media defined in imported CSS files', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	expect(css).not.toContain('@custom-media');
	expect(css).not.toMatch(/\(--\w+\)/);
	expect(css).toContain('@media (width >= 800px)');
	expect(css).toContain('@media (width <= 500px)');
	expect(css).toContain('@media (width >= 1000px)');
	// The definition from breakpoints.css overrides the one it imports.
	expect(css).not.toContain('600px');
});
