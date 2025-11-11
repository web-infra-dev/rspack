
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["362"],
	{
		803: function (e, t, n) {
			(n.r(t), n.d(t, { default: () => l, loadApp: () => r }));
			var o = n(740);
			const a = new o.A({
					target: document.querySelector("#app_04"),
					props: { name: "world" }
				}),
				r = e =>
					new o.A({
						target: document.querySelector("#app_04"),
						props: { name: "world" }
					});
			window.app = a;
			const l = a;
		},
		740: function (e, t, n) {
			n.d(t, { A: () => p });
			var o = n(373);
			function a(e) {
				let t, n, a, r;
				return {
					c() {
						((t = (0, o.ND4)("h1")),
							(n = (0, o.Qq7)("Hello From Svelte ")),
							(a = (0, o.Qq7)(e[0])),
							(r = (0, o.Qq7)("!")),
							(0, o.CFu)(t, "class", "svelte-1ucbz36"));
					},
					m(e, l) {
						((0, o.Yry)(e, t, l),
							(0, o.BCw)(t, n),
							(0, o.BCw)(t, a),
							(0, o.BCw)(t, r));
					},
					p(e, [t]) {
						1 & t && (0, o.iQh)(a, e[0]);
					},
					i: o.lQ1,
					o: o.lQ1,
					d(e) {
						e && (0, o.YoD)(t);
					}
				};
			}
			function r(e, t, n) {
				let { name: o } = t;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && n(0, (o = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && n(0, (o = e.name));
					}),
					[o]
				);
			}
			(n(176), n(113));
			class l extends o.r7T {
				constructor(e) {
					(super(), (0, o.TsN)(this, e, r, a, o.jXN, { name: 0 }));
				}
			}
			const p = l;
		},
		113: function (e, t, n) {
			e.exports = {};
		}
	}
]);
