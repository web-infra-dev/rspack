import './style.css?targets';
import './style.css?no-draft';
import './style.css?exclude';
import './style.css?no-targets';
import './deep.css';

const fs = require('fs');
const path = require('path');

it('should only expand custom media when the draft and transform are enabled', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	// Preserved by `?no-draft`, `?exclude`, and `?no-targets`.
	expect(css.match(/@custom-media --wide/g)).toHaveLength(3);
	expect(css).toContain('.parent >>> .child');
});
