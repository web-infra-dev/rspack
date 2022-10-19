function a() {}

function b() {
	c(a);
	c(a);
}

export { a, b, c, dddd };

export function c() {
	class Test {
		a() {
			a();
		}
	}
	function result() {}
}
const dddd = c;
console.log(a);
// export default 20;
