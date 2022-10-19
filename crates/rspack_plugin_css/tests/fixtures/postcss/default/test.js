function a() {}

function b() {}

export { a, b, c };

function c() {
	class Test {
		a() {
			a();
		}
	}
	function result() {}
}

console.log(a);
// export default 20;
