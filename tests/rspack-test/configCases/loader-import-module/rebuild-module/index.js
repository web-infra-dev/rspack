import value from './a.js';

it('should import a new module via importModule during rebuildModule', () => {
	// the first loader run imports lib1.js, the rebuild run imports lib2.js
	expect(value).toBe(2);
});
