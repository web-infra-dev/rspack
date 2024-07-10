import csvToMarkdown from "csv-to-markdown-table";
import fs from "fs-extra";

import {
	ECompareResultType,
	ECompilerType,
	type ITestReporter,
	type TModuleCompareResult
} from "../type";

export interface IDiffStatsReporterOptions {
	header?: string[];
	footer?: string[];
	file: string;
	report?: boolean;
}

export type TCompilerTypeId =
	| ECompilerType.Rspack
	| ECompilerType.Webpack
	| "common";
export type TModuleTypeId = "normal" | "runtime";
export type TDimenTypeId = "modules" | "lines" | "lines-in-common";
export type TCaseSummaryId =
	`${TCompilerTypeId}|${TModuleTypeId}|${TDimenTypeId}`;
export type TCaseSummary = Record<TCaseSummaryId, number>;

const toPercent = (d: number) => (d * 100).toFixed(2) + "%";
const toFirstLetterUpperCase = (s: string) =>
	(s.charAt(0).toUpperCase() + s.slice(1)).split("-").join(" ");
const GITHUB_RUN_ID = process.env.GITHUB_RUN_ID;

export class DiffStatsReporter
	implements ITestReporter<TModuleCompareResult[]>
{
	private summary: Map<string, TCaseSummary> = new Map();
	private failed: Set<string> = new Set();

	constructor(private options: IDiffStatsReporterOptions) {}
	async init(data: TModuleCompareResult[] = []) {}
	async failure(id: string) {
		this.failed.add(id);
		if (this.summary.has(id)) {
			this.summary.delete(id);
		}
	}
	async increment(id: string, data: TModuleCompareResult[]) {
		if (this.failed.has(id)) return;
		if (!this.summary.has(id)) {
			this.summary.set(id, this.createSummary());
		}
		const current = this.summary.get(id)!;

		for (let item of data) {
			if (item.type === ECompareResultType.Missing) continue;
			const moduleType: TModuleTypeId = item.name.startsWith("webpack/runtime")
				? "runtime"
				: "normal";
			// handle modules
			if (item.type === ECompareResultType.OnlySource) {
				current[`${ECompilerType.Rspack}|${moduleType}|${"modules"}`]++;
			} else if (item.type === ECompareResultType.OnlyDist) {
				current[`${ECompilerType.Webpack}|${moduleType}|${"modules"}`]++;
			} else {
				current[`${"common"}|${moduleType}|${"modules"}`]++;
			}
			// handle lines
			current[`${ECompilerType.Rspack}|${moduleType}|${"lines"}`] +=
				item.lines?.source || 0;
			current[`${ECompilerType.Webpack}|${moduleType}|${"lines"}`] +=
				item.lines?.dist || 0;
			current[`${"common"}|${moduleType}|${"lines"}`] +=
				item.lines?.common || 0;
			// handle lines in common modules
			if (
				item.type === ECompareResultType.Same ||
				item.type === ECompareResultType.Different
			) {
				current[`${ECompilerType.Rspack}|${moduleType}|${"lines-in-common"}`] +=
					item.lines?.source || 0;
				current[
					`${ECompilerType.Webpack}|${moduleType}|${"lines-in-common"}`
				] += item.lines?.dist || 0;
				current[`${"common"}|${moduleType}|${"lines-in-common"}`] +=
					item.lines?.common || 0;
			}
		}
	}
	async output() {
		const chunks: string[] = [];
		for (let [id, summary] of this.summary.entries()) {
			chunks.push(this.stringifySummary(id, summary));
		}
		for (let id of this.failed.values()) {
			chunks.push(`### ${id}\n\n> Failed\n\n`);
		}
		const output = [
			...(this.options.header || []),
			chunks.join("\n---\n"),
			...(this.options.footer || [])
		].join("\n\n");
		fs.ensureFileSync(this.options.file);
		fs.writeFileSync(this.options.file, output);
	}
	private stringifySummary(id: string, summary: TCaseSummary) {
		let output = `### ${id}\n\n`;
		for (let moduleType of ["runtime", "normal"] as TModuleTypeId[]) {
			const csv: string[] = [];
			csv.push(
				`${
					moduleType.charAt(0).toUpperCase() + moduleType.slice(1)
				} Modules,Rspack Only,Common,Webpack Only,Common Percent`
			);
			for (let dimen of [
				"modules",
				"lines",
				"lines-in-common"
			] as TDimenTypeId[]) {
				const counts = (
					[
						ECompilerType.Rspack,
						"common",
						ECompilerType.Webpack
					] as TCompilerTypeId[]
				).map(i => summary[`${i}|${moduleType}|${dimen}`]);
				csv.push(
					`${
						dimen === "lines-in-common"
							? "Lines(Common Modules)"
							: toFirstLetterUpperCase(dimen)
					},${counts.join(",")},${toPercent(
						counts[1] / (counts[0] + counts[1] + counts[2])
					)}`
				);
			}
			output += csvToMarkdown(csv.join("\n"), ",", true);
			output += "\n\n";
		}
		if (this.options.report && GITHUB_RUN_ID) {
			output += `> [View diff report](https://web-infra-dev.github.io/rspack-report-website/diff/${GITHUB_RUN_ID}/diff_${id}.html)`;
			output += "\n\n";
		}
		return output;
	}
	private createSummary(): TCaseSummary {
		let result: Partial<TCaseSummary> = {};
		for (let i of [
			ECompilerType.Rspack,
			ECompilerType.Webpack,
			"common"
		] as TCompilerTypeId[]) {
			for (let j of ["runtime", "normal"] as TModuleTypeId[]) {
				for (let k of [
					"modules",
					"lines",
					"lines-in-common"
				] as TDimenTypeId[]) {
					result[`${i}|${j}|${k}`] = 0;
				}
			}
		}
		return result as TCaseSummary;
	}
}
