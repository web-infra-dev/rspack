const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	prod
		? __non_webpack_require__("./249.bundle1.js")
		: __non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toEqual({
				class: prod ? "my-app-491-S" : "style_module_css-class",
				local: prod
					? "my-app-491-Zw my-app-491-yl my-app-491-J_ my-app-491-gc"
					: "style_module_css-local1 style_module_css-local2 style_module_css-local3 style_module_css-local4",
				local2: prod
					? "my-app-491-Xg my-app-491-AY"
					: "style_module_css-local5 style_module_css-local6",
				nested: prod
					? "my-app-491-RX my-app-491-X2"
					: "style_module_css-nested1 style_module_css-nested3",
				ident: prod ? "my-app-491-yR" : "style_module_css-ident",
				keyframes: prod ? "my-app-491-y3" : "style_module_css-localkeyframes",
				animation: prod ? "my-app-491-oQ" : "style_module_css-animation",
				vars: prod
					? "my-app-491-gR my-app-491-xk"
					: "style_module_css-vars style_module_css-globalVars"
			});
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
