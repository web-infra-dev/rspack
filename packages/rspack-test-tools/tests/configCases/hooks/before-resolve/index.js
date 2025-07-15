import { text } from "./text.txt";

it("should compile successfully when resolve data is changed ", () => {
	expect(text.trim()).toBe("Hello World");
});
