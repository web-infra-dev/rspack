import { value } from "./dep";
import { value as aValue } from "./a";
import { value as bValue } from "./b";

let acceptedDep;
let acceptedArray;

if (import.meta.hot) {
	import.meta.hot.accept("./dep", mod => {
		acceptedDep = mod;
	});
	import.meta.hot.accept(["./a", "./b"], mods => {
		acceptedArray = mods;
	});
}

it("passes updated namespaces to Vite accept callbacks", async () => {
	expect(value).toBe(1);
	expect(aValue).toBe("a1");
	expect(bValue).toBe("b1");

	await NEXT_HMR();

	expect(acceptedDep.value).toBe(2);
	expect(acceptedArray.map(mod => mod && mod.value)).toEqual(["a2", "b2"]);
	expect(value).toBe(2);
	expect(aValue).toBe("a2");
	expect(bValue).toBe("b2");
});
