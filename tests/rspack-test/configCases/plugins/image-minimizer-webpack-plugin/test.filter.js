module.exports = () => {
	try {
		require("image-minimizer-webpack-plugin");
		require("sharp");
		return true;
	} catch {
		return "image-minimizer-webpack-plugin or sharp is not installed";
	}
};
