import { bar } from './shared.js';
import fs from 'fs'

bar;
it('foo', () => {
	const files = fs.readdirSync(__dirname);
	const sharedChunks = files.filter(filename => filename.startsWith('shared-shared_js')).length
	expect(sharedChunks).toBe(2);
});
