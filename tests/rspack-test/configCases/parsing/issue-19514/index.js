it("should compile and work", () => {
	function main() {
		if (!import.meta.webpackHot) {
			return;
		}
		if (import.meta.webpackHot.status() !== "idle") {
			console.log("idle");
		}
	}
	main();
});
