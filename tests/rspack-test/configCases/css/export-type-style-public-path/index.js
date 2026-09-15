import './style.css';
import text from './changed.css?text';
import sheet from './changed.css?css-style-sheet';

const getCss = () => Array.from(document.getElementsByTagName('style'), style => style.textContent).join('\n');

it('should use the runtime public path only when enabled', () => {
	const css = getCss();
	expect(css).toContain(`url("${ASSET_PREFIX}assets/image.svg")`);
	expect(css).toContain('url("https://fixed.example.com/assets/image.svg")');
	expect(css).toContain('url("data:image/svg+xml;base64,');
	expect(css).toContain('url("https://external.example.com/image.svg")');
	expect(css).not.toContain('emitted/');
	if (RUNTIME_PUBLIC_PATH) expect(css).not.toContain('/build-time/');
	expect(css).not.toContain('__RSPACK_PLUGIN_CSS_');
});

it('should resolve imported CSS asset URLs with the runtime public path', () => {
	const css = getCss();
	expect(css).toContain('.imported');
	expect(css).toContain('@layer');
	expect(css.split(`url("${ASSET_PREFIX}assets/image.svg")`).slice(1)).toHaveLength(3);
});

it('should use the runtime public path in lazy CSS', async () => {
	await import('./lazy.css');
	const css = getCss();
	expect(css).toContain('.lazy');
	expect(css.split(`url("${ASSET_PREFIX}assets/image.svg")`).slice(1)).toHaveLength(4);
});

it('should observe public path changes before evaluating another CSS module', () => {
	const publicPath = __webpack_public_path__;
	try {
		__webpack_public_path__ = 'https://test.cases/changed/';
		require('./changed.css');
		expect(getCss()).toContain(`url("${RUNTIME_PUBLIC_PATH ? "https://test.cases/changed/" : ASSET_PREFIX}assets/image.svg")`);
	} finally {
		__webpack_public_path__ = publicPath;
	}
});

it('should leave text and stylesheet exports unchanged when enabled', () => {
	expect(text).toContain(`url("${BUILD_PREFIX}assets/image.svg")`);
	const css = Array.from(sheet.cssRules, rule => rule.cssText).join('\n');
	expect(css).toContain(`${BUILD_PREFIX}assets/image.svg`);
	expect(text).not.toContain('https://test.cases/path/');
	expect(css).not.toContain('https://test.cases/path/');
});
