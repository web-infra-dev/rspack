export function loadConstant() {
	return new Promise(resolve => {
		require("exports".split(","), resolve);
	});
}
