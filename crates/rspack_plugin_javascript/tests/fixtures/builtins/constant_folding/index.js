if (process.env.NODE_ENV === "development") {
	require("./development");
} else {
	require("./production");
}
