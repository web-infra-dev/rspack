export let then;
export { value } from "./value.js";

await Promise.resolve();
Promise.resolve().then(() => {
	then = () => {
		throw new Error("unexpected then");
	};
});
