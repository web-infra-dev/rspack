it("should modify runtime module source in main", async () => {
	await import("./index.css");
	expect(__webpack_require__.f.miniCss.test).toBeTruthy();	
});
