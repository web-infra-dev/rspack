/*
 * This file was automatically generated.
 * DO NOT MODIFY BY HAND.
 * Run `yarn special-lint-fix` to update
 */
"use strict";
function n(
	t,
	{
		instancePath: l = "",
		parentData: e,
		parentDataProperty: s,
		rootData: a = t
	} = {}
) {
	let r = null,
		o = 0;
	const u = o;
	let i = !1;
	const p = o;
	if (o === p)
		if (Array.isArray(t)) {
			const n = t.length;
			for (let l = 0; l < n; l++) {
				let n = t[l];
				const e = o,
					s = o;
				let a = !1,
					u = null;
				const i = o,
					p = o;
				let f = !1;
				const h = o;
				if (!(n instanceof RegExp)) {
					const n = { params: {} };
					null === r ? (r = [n]) : r.push(n), o++;
				}
				var c = h === o;
				if (((f = f || c), !f)) {
					const t = o;
					if (o === t)
						if ("string" == typeof n) {
							if (n.length < 1) {
								const n = { params: {} };
								null === r ? (r = [n]) : r.push(n), o++;
							}
						} else {
							const n = { params: { type: "string" } };
							null === r ? (r = [n]) : r.push(n), o++;
						}
					(c = t === o), (f = f || c);
				}
				if (f) (o = p), null !== r && (p ? (r.length = p) : (r = null));
				else {
					const n = { params: {} };
					null === r ? (r = [n]) : r.push(n), o++;
				}
				if ((i === o && ((a = !0), (u = 0)), a))
					(o = s), null !== r && (s ? (r.length = s) : (r = null));
				else {
					const n = { params: { passingSchemas: u } };
					null === r ? (r = [n]) : r.push(n), o++;
				}
				if (e !== o) break;
			}
		} else {
			const n = { params: { type: "array" } };
			null === r ? (r = [n]) : r.push(n), o++;
		}
	var f = p === o;
	if (((i = i || f), !i)) {
		const n = o,
			l = o;
		let e = !1;
		const s = o;
		if (!(t instanceof RegExp)) {
			const n = { params: {} };
			null === r ? (r = [n]) : r.push(n), o++;
		}
		var h = s === o;
		if (((e = e || h), !e)) {
			const n = o;
			if (o === n)
				if ("string" == typeof t) {
					if (t.length < 1) {
						const n = { params: {} };
						null === r ? (r = [n]) : r.push(n), o++;
					}
				} else {
					const n = { params: { type: "string" } };
					null === r ? (r = [n]) : r.push(n), o++;
				}
			(h = n === o), (e = e || h);
		}
		if (e) (o = l), null !== r && (l ? (r.length = l) : (r = null));
		else {
			const n = { params: {} };
			null === r ? (r = [n]) : r.push(n), o++;
		}
		(f = n === o), (i = i || f);
	}
	if (!i) {
		const t = { params: {} };
		return null === r ? (r = [t]) : r.push(t), o++, (n.errors = r), !1;
	}
	return (
		(o = u),
		null !== r && (u ? (r.length = u) : (r = null)),
		(n.errors = r),
		0 === o
	);
}
function t(
	l,
	{
		instancePath: e = "",
		parentData: s,
		parentDataProperty: a,
		rootData: r = l
	} = {}
) {
	let o = null,
		u = 0;
	const i = u;
	let p = !1;
	const c = u;
	if (u === c)
		if ("string" == typeof l) {
			if (l.length < 1) {
				const n = { params: {} };
				null === o ? (o = [n]) : o.push(n), u++;
			}
		} else {
			const n = { params: { type: "string" } };
			null === o ? (o = [n]) : o.push(n), u++;
		}
	var f = c === u;
	if (((p = p || f), !p)) {
		const t = u;
		if (u === t)
			if (l && "object" == typeof l && !Array.isArray(l)) {
				let t;
				if (void 0 === l.banner && (t = "banner")) {
					const n = { params: { missingProperty: t } };
					null === o ? (o = [n]) : o.push(n), u++;
				} else {
					const t = u;
					for (const n in l)
						if (
							"banner" !== n &&
							"entryOnly" !== n &&
							"exclude" !== n &&
							"footer" !== n &&
							"include" !== n &&
							"raw" !== n &&
							"test" !== n
						) {
							const t = { params: { additionalProperty: n } };
							null === o ? (o = [t]) : o.push(t), u++;
							break;
						}
					if (t === u) {
						if (void 0 !== l.banner) {
							let n = l.banner;
							const t = u,
								e = u;
							let s = !1;
							const a = u;
							if ("string" != typeof n) {
								const n = { params: { type: "string" } };
								null === o ? (o = [n]) : o.push(n), u++;
							}
							var h = a === u;
							if (((s = s || h), !s)) {
								const t = u;
								if (!(n instanceof Function)) {
									const n = { params: {} };
									null === o ? (o = [n]) : o.push(n), u++;
								}
								(h = t === u), (s = s || h);
							}
							if (s) (u = e), null !== o && (e ? (o.length = e) : (o = null));
							else {
								const n = { params: {} };
								null === o ? (o = [n]) : o.push(n), u++;
							}
							var y = t === u;
						} else y = !0;
						if (y) {
							if (void 0 !== l.entryOnly) {
								const n = u;
								if ("boolean" != typeof l.entryOnly) {
									const n = { params: { type: "boolean" } };
									null === o ? (o = [n]) : o.push(n), u++;
								}
								y = n === u;
							} else y = !0;
							if (y) {
								if (void 0 !== l.exclude) {
									const t = u,
										s = u;
									let a = !1,
										i = null;
									const p = u;
									if (
										(n(l.exclude, {
											instancePath: e + "/exclude",
											parentData: l,
											parentDataProperty: "exclude",
											rootData: r
										}) ||
											((o = null === o ? n.errors : o.concat(n.errors)),
											(u = o.length)),
										p === u && ((a = !0), (i = 0)),
										a)
									)
										(u = s), null !== o && (s ? (o.length = s) : (o = null));
									else {
										const n = { params: { passingSchemas: i } };
										null === o ? (o = [n]) : o.push(n), u++;
									}
									y = t === u;
								} else y = !0;
								if (y) {
									if (void 0 !== l.footer) {
										const n = u;
										if ("boolean" != typeof l.footer) {
											const n = { params: { type: "boolean" } };
											null === o ? (o = [n]) : o.push(n), u++;
										}
										y = n === u;
									} else y = !0;
									if (y) {
										if (void 0 !== l.include) {
											const t = u,
												s = u;
											let a = !1,
												i = null;
											const p = u;
											if (
												(n(l.include, {
													instancePath: e + "/include",
													parentData: l,
													parentDataProperty: "include",
													rootData: r
												}) ||
													((o = null === o ? n.errors : o.concat(n.errors)),
													(u = o.length)),
												p === u && ((a = !0), (i = 0)),
												a)
											)
												(u = s),
													null !== o && (s ? (o.length = s) : (o = null));
											else {
												const n = { params: { passingSchemas: i } };
												null === o ? (o = [n]) : o.push(n), u++;
											}
											y = t === u;
										} else y = !0;
										if (y) {
											if (void 0 !== l.raw) {
												const n = u;
												if ("boolean" != typeof l.raw) {
													const n = { params: { type: "boolean" } };
													null === o ? (o = [n]) : o.push(n), u++;
												}
												y = n === u;
											} else y = !0;
											if (y)
												if (void 0 !== l.test) {
													const t = u,
														s = u;
													let a = !1,
														i = null;
													const p = u;
													if (
														(n(l.test, {
															instancePath: e + "/test",
															parentData: l,
															parentDataProperty: "test",
															rootData: r
														}) ||
															((o = null === o ? n.errors : o.concat(n.errors)),
															(u = o.length)),
														p === u && ((a = !0), (i = 0)),
														a)
													)
														(u = s),
															null !== o && (s ? (o.length = s) : (o = null));
													else {
														const n = { params: { passingSchemas: i } };
														null === o ? (o = [n]) : o.push(n), u++;
													}
													y = t === u;
												} else y = !0;
										}
									}
								}
							}
						}
					}
				}
			} else {
				const n = { params: { type: "object" } };
				null === o ? (o = [n]) : o.push(n), u++;
			}
		if (((f = t === u), (p = p || f), !p)) {
			const n = u;
			if (!(l instanceof Function)) {
				const n = { params: {} };
				null === o ? (o = [n]) : o.push(n), u++;
			}
			(f = n === u), (p = p || f);
		}
	}
	if (!p) {
		const n = { params: {} };
		return null === o ? (o = [n]) : o.push(n), u++, (t.errors = o), !1;
	}
	return (
		(u = i),
		null !== o && (i ? (o.length = i) : (o = null)),
		(t.errors = o),
		0 === u
	);
}
(module.exports = t), (module.exports.default = t);
