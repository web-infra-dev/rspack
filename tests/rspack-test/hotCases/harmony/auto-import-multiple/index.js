import { value } from "./file";
import value2 from "./commonjs";

it("should auto-import multiple ES6 imported values on accept", async () => {
	expect(value).toBe(1);
	expect(value2).toBe(10);
	await NEXT_HMR();
	expect(value).toBe(2);
	expect(value2).toBe(20);
	outside();
});

function outside() {
	expect(value).toBe(2);
	expect(value2).toBe(20);
}

module.hot.accept(["./file", "./commonjs"]);