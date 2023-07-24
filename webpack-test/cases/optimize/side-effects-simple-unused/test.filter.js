
module.exports = function(config) {
	console.log(config)
	return config.mode !== "development";
};

// module.exports = () => {return false}


