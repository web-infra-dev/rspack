import fs from 'fs'
import path from 'path'

it('should properly quote export names that are not valid identifiers', async () => {
const json = fs.readFileSync(path.resolve(import.meta.dirname, './json.mjs'), 'utf-8');

// The export statement should quote special identifiers
expect(json).toContain('"!top"');
expect(json).toContain('"a.b"');
expect(json).toContain('"a-b"');

// Should not contain unquoted special chars which would be syntax errors
expect(json).not.toMatch(/\bas !top\b/);
expect(json).not.toMatch(/\bas a\.b\b/);

const jsonModule = await import(/*webpackIgnore: true*/'./json.mjs');
expect(jsonModule["!top"]).toBe("exclamation");
expect(jsonModule["regular"]).toBe("normal");
expect(jsonModule["a.b"]).toBe("dot");
expect(jsonModule["a-b"]).toBe("dash");
})
