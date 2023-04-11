module.exports = {
	builtins: {
		html: [
			{
				templateContent:
					"<!DOCTYPE html><html><body><div><%= env %></div></body></html>",
				templateParameters: {
					env: "production"
				}
			}
		]
	}
};
