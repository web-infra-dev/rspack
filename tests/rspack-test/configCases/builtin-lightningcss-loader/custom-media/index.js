import * as styles from './index.module.css';

const fs = require('fs');
const path = require('path');

it('should expand custom media queries in the loader', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	expect(css).not.toContain('@custom-media');
	expect(css).not.toContain('(--');
	expect(css).toContain('@media (width >= 800px)');
	expect(css).toContain('@media (width >= 800px) and (orientation: landscape)');
	expect(css).toContain('@media (width <= 500px)');
	expect(css).toContain('@media (width >= 900px)');
});

it('should keep CSS Modules exports', () => {
	const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');

	for (const name of ['root', 'wide', 'compact', 'escaped']) {
		expect(typeof styles[name]).toBe('string');
		expect(css).toContain(`.${styles[name]}`);
	}
});

it('should emit a CSS source map', () => {
	const map = JSON.parse(
		fs.readFileSync(path.resolve(__dirname, 'bundle0.css.map'), 'utf-8'),
	);

	expect(map.sources.some((source) => source.endsWith('index.module.css'))).toBe(true);
});
