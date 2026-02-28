import { dir, dir2, file } from './file.js'

it("should generate correct __dirname", () => {
	expect(dir).toMatch(/[\\/]node-shims-in-concatenated-module$/);
	expect(dir2).toMatch(/[\\/]node-shims-in-concatenated-module\/$/);
});

it("should generate correct __filename", () => {
	expect(file).toMatch(/[\\/]main.mjs$/);
});
