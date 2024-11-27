const context = require.context("./", false, /mod/);

console.log(context);

document.getElementById("root").textContent = "__PAGE_RENDER__";

if (module.hot) {
	module.hot.accept();
	module.hot.addStatusHandler(status => {
		if (status === "idle") {
			document.getElementById("root").textContent = "__HMR_UPDATED__";
		}
	});
}
