function main() {
	return "test-keep-fn-names";
}

function callMain() {
	const call = main();

	console.log(call);
	return call;
}

main();

callMain();
