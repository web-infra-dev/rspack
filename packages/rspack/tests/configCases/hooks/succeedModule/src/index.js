import { answer } from "./answer";

try {
	require(`./child/${a}.js`);
} catch (e) {}
console.log(`hello ${answer}`);
it("should work", () => {});
