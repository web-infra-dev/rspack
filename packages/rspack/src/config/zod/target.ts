import { z } from "zod";

const allowTarget = z
	.enum([
		"web",
		"webworker",
		"es3",
		"es5",
		"es2015",
		"es2016",
		"es2017",
		"es2018",
		"es2019",
		"es2020",
		"es2021",
		"es2022",
		"browserslist"
	])
	.or(z.literal("node"))
	.or(z.literal("async-node"))
	.or(
		z.custom<`node${number}`>(
			value => typeof value === "string" && /^node\d+$/.test(value)
		)
	)
	.or(
		z.custom<`async-node${number}`>(
			value => typeof value === "string" && /^async-node\d+$/.test(value)
		)
	)
	.or(
		z.custom<`node${number}.${number}`>(
			value => typeof value === "string" && /^node\d+\.\d+$/.test(value)
		)
	)
	.or(
		z.custom<`async-node${number}.${number}`>(
			value => typeof value === "string" && /^async-node\d+\.\d+$/.test(value)
		)
	)
	.or(z.literal("electron-main"))
	.or(
		z.custom<`electron${number}-main`>(
			value => typeof value === "string" && /^electron\d+-main$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-main`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-main$/.test(value)
		)
	)
	.or(z.literal("electron-renderer"))
	.or(
		z.custom<`electron${number}-renderer`>(
			value => typeof value === "string" && /^electron\d+-renderer$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-renderer`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-renderer$/.test(value)
		)
	)
	.or(z.literal("electron-preload"))
	.or(
		z.custom<`electron${number}-preload`>(
			value => typeof value === "string" && /^electron\d+-preload$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-preload`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-preload$/.test(value)
		)
	);

export function target() {
	return z.literal(false).or(allowTarget).or(allowTarget.array());
}
