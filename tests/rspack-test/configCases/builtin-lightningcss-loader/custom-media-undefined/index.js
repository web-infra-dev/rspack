import './definitions.css';
import './component.css';

const fs = require('fs');
const path = require('path');

it('should leave custom media unchanged in a file that uses an undefined name', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	expect(css).toContain('@custom-media --narrow');
	expect(css).toContain('@media (--narrow)');
	expect(css).toContain('@media (--wide)');
	// definitions.css is transformed on its own, so its definition is removed.
	expect(css).not.toContain('@custom-media --wide');
});
