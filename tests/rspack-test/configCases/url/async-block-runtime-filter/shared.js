export function loadAsset() {
	return new Promise((resolve, reject) => {
		require.ensure([], require => {
			const outer = new URL("./asset.txt", import.meta.url);
			require.ensure([], require => {
				resolve([
					outer.pathname,
					new URL("./nested.txt", import.meta.url).pathname,
					new URL("./target.js", import.meta.url).pathname
				]);
			}, error => reject(error));
		}, error => reject(error));
	});
}
export function value() {
	return "unused runtime";
}
