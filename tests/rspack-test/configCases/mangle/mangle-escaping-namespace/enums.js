export const ENUM_A = "a-value";
export const ENUM_B = "b-value";
export const NUM = 42;
export let currentValue = 0;
export function increment() {
	currentValue++;
}
const unusual = "unusual";
export { unusual as "not-an-identifier" };
export default "default-value";
export const canMangle = __webpack_exports_info__.ENUM_A.canMangle;
