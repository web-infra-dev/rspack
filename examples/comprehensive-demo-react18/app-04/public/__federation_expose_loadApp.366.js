
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["366"],
	{
		995: function (e, t, n) {
			(n.r(t), n.d(t, { default: () => o }));
			var l = n(740);
			const o = (e, t) =>
				new l.A({
					target: document.querySelector(`#${e}`),
					props: { name: t }
				});
		},
		740: function (e, t, n) {
			n.d(t, { A: () => a });
			var l = n(373);
			function o(e) {
				let t, n, o, r;
				return {
					c() {
						((t = (0, l.ND4)("h1")),
							(n = (0, l.Qq7)("Hello From Svelte ")),
							(o = (0, l.Qq7)(e[0])),
							(r = (0, l.Qq7)("!")),
							(0, l.CFu)(t, "class", "svelte-1ucbz36"));
					},
					m(e, s) {
						((0, l.Yry)(e, t, s),
							(0, l.BCw)(t, n),
							(0, l.BCw)(t, o),
							(0, l.BCw)(t, r));
					},
					p(e, [t]) {
						1 & t && (0, l.iQh)(o, e[0]);
					},
					i: l.lQ1,
					o: l.lQ1,
					d(e) {
						e && (0, l.YoD)(t);
					}
				};
			}
			function r(e, t, n) {
				let { name: l } = t;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && n(0, (l = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && n(0, (l = e.name));
					}),
					[l]
				);
			}
			(n(176), n(113));
			class s extends l.r7T {
				constructor(e) {
					(super(), (0, l.TsN)(this, e, r, o, l.jXN, { name: 0 }));
				}
			}
			const a = s;
		},
		113: function (e, t, n) {
			e.exports = {};
		}
	}
]);
