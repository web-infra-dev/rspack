import './style.css?targets-css';
import './style.css?targets-auto';
import './style.css?targets-global';
import './style.css?targets-module';
import './style.css?no-draft';
import './style.css?exclude';
import './style.css?no-targets';
import './deep.css';

const fs = require('fs');
const path = require('path');

it('should only expand custom media when manually enabled for builtin CSS', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	// Target-based transforms are disabled by default for builtin CSS. The
	// explicit `include.customMediaQueries` on deep.css still enables it.
	expect(css.match(/@custom-media --wide/g)).toHaveLength(7);
	expect(css).toContain('.parent >>> .child');
});
