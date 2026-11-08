import(/* webpackChunkName: "shared" */ "./shared").then(({ value }) => {
	globalThis.__HMR_PROCESS_ASSETS_B__ = value;
});

module.hot.accept();
