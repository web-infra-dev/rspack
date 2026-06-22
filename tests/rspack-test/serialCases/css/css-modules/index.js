const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	prod
		? require("./249.bundle1.js")
		: require("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toEqual({
				// TODO: enable when missing CSS module exports can be reported as warnings.
				// global: undefined,
				class: prod ? "my-app-491-S" : "./style.module.css-class",
				local: prod
					? "my-app-491-Zw my-app-491-yl my-app-491-J_ my-app-491-gc"
					: "./style.module.css-local1 ./style.module.css-local2 ./style.module.css-local3 ./style.module.css-local4",
				local2: prod
					? "my-app-491-Xg my-app-491-AY"
					: "./style.module.css-local5 ./style.module.css-local6",
				// TODO: include the missing nested2 export when it can be reported as a warning.
				// nested: prod
				// 	? "my-app-491-RX undefined my-app-491-X2"
				// 	: "./style.module.css-nested1 undefined ./style.module.css-nested3",
				nested: prod
					? "my-app-491-RX my-app-491-X2"
					: "./style.module.css-nested1 ./style.module.css-nested3",
				ident: prod ? "my-app-491-yR" : "./style.module.css-ident",
				keyframes: prod ? "my-app-491-y3" : "./style.module.css-localkeyframes",
				animation: prod ? "my-app-491-oQ" : "./style.module.css-animation",
				// TODO: include local-color/global-color when CSS custom property exports are supported.
				// vars: prod
				// 	? "--my-app-491-y4 my-app-491-gR undefined my-app-491-xk"
				// 	: "--./style.module.css-local-color ./style.module.css-vars undefined ./style.module.css-globalVars",
				vars: prod
					? "my-app-491-gR my-app-491-xk"
					: "./style.module.css-vars ./style.module.css-globalVars"
			});
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
