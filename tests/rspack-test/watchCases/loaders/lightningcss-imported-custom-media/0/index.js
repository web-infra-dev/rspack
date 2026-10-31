import { readFileSync } from 'fs';
import { join } from 'path';
import './component.css';

it('should rebuild a module when the custom media it imports changes', () => {
	const width = WATCH_STEP === '0' ? '800px' : '900px';
	const css = readFileSync(join(__dirname, 'style.css'), 'utf-8');
	expect(css).not.toContain('@custom-media');
	expect(css).toContain(`@media (width >= ${width})`);
});
