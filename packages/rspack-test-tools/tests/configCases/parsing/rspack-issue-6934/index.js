import fs from 'fs';
import { A } from './b.js';
it("should not generate duplicated harmony exports when using named exports and named exports from", () => {
	A;
	let file = fs.readFileSync(__filename, "utf-8")
	expect(file.split(`A: function() { return /* reexport safe */`)).toHaveLength(3)
})
