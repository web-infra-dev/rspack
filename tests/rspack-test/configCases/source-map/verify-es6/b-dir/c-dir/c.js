export function c0() {
	c1("*c0*");
}
function c1() {
	c2("*c1*");
}
function c2() {
	throw new Error("*c2*");
}
