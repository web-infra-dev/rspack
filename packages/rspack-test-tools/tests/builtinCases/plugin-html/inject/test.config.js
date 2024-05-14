module.exports = {
	builtins: {
		html: [
			{
				filename: "inject_head.html",
				inject: "head"
			},
			{
				filename: "inject_body.html",
				inject: "body"
			},
			{
				filename: "inject_false.html",
				inject: false
			}
		]
	}
};
