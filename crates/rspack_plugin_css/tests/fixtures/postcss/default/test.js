function a() {}

function b() {
	c(a);
	c(a);
}

export { a, b, c };

export function c() {
	class Test {
		a() {
			a();
		}
	}
	function result() {}
}
export const dddd = c;
// export default 20;
