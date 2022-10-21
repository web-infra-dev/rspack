function a() {}

function b() {
	c(a);
	c(a);
}

export { a, b };

export function c() {
	class Test {
		a() {
			a();
		}
	}
	function result() {}
}
export const dddd = c;
dddd;
c;
// export default 20;
