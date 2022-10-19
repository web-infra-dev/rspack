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

console.log(a);
// export default 20;
