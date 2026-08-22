import * as uiLib from 'ui-lib';
import * as uiLibDep from 'ui-lib-dep';

const fs = require('fs');
const path = require('path');

const sharedDir = path.join(__dirname, 'independent-packages');
const uiLibEntry = path.join(sharedDir, 'ui_lib/1.0.0', 'share-entry.mjs');
const uiLibDepEntry = path.join(sharedDir, 'ui_lib_dep/1.0.0', 'share-entry.mjs');

const read = p => fs.readFileSync(p, 'utf-8');

it('should consume the shared modules in the host bundle', () => {
	expect(uiLib.Badge).toBe('Badge');
	expect(uiLib.MessagePro).toBe('MessagePro');
	expect(uiLibDep.Message).toBe('Message');
});

it('should emit module-type fallback containers (no `Library name must be unset`)', () => {
	// Before the fix the `library: { type: 'module' }` container failed to
	// compile, so no fallback file was emitted.
	expect(fs.existsSync(uiLibEntry)).toBe(true);
	expect(fs.existsSync(uiLibDepEntry)).toBe(true);
});

it('should emit ESM containers exposing the federation get/init interface', () => {
	for (const entry of [uiLibEntry, uiLibDepEntry]) {
		const code = read(entry);
		expect(code).toMatch(/export\s*\{[^}]*\bas get\b/);
		expect(code).toMatch(/export\s*\{[^}]*\bas init\b/);
	}
});

it('should tree-shake the ESM containers down to the used exports', () => {
	// ui-lib-dep: usedExports ['Message'] → keep Message, drop Text/Spin.
	const uiLibDepCode = read(uiLibDepEntry);
	expect(uiLibDepCode).toContain('Message:"Message"');
	expect(uiLibDepCode).not.toContain('Text:"Text"');
	expect(uiLibDepCode).not.toContain('Spin:"Spin"');

	// ui-lib: usedExports ['Badge','MessagePro'] → drop Button/List/SpinPro.
	const uiLibCode = read(uiLibEntry);
	expect(uiLibCode).toContain('Badge:"Badge"');
	expect(uiLibCode).not.toContain('Button:"Button"');
	expect(uiLibCode).not.toContain('List:"List"');
});
