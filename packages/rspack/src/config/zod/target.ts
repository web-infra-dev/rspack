import { z } from "zod";

const allowTarget = z.enum([
	"node",
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
]);

export function target() {
	return z.literal(false).or(allowTarget).or(allowTarget.array());
}
