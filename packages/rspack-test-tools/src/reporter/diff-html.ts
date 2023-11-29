import {
	ITestReporter,
	TDiffStats,
	TDiffStatsItem,
	TModuleCompareResult
} from "../type";
import fs from "fs-extra";
import path from "path";

const VIEWER_DIR = path.join(__dirname, "../viewer");
const DIFF_STATS_PLACEHOLDER = "$$RSPACK_DIFF_STATS_PLACEHOLDER$$";
const DEFAULT_IGNORE = /node_modules/;

export interface IDiffHtmlReporterOptions {
	dist: string;
	ignore?: RegExp;
}

export class DiffHtmlReporter implements ITestReporter<TModuleCompareResult[]> {
	private failed: Set<string> = new Set();
	private results: Map<string, TDiffStatsItem[]> = new Map();

	constructor(private options: IDiffHtmlReporterOptions) {}

	async init(data: TModuleCompareResult[] = []) {}
	async failure(id: string) {
		this.failed.add(id);
		this.results.delete(id);
	}
	async increment(id: string, data: TModuleCompareResult[]) {
		if (this.failed.has(id)) return;
		if (!this.results.has(id)) {
			this.results.set(id, []);
		}
		const ignore = this.options.ignore || DEFAULT_IGNORE;
		const current = this.results.get(id)!;
		for (let i of data) {
			if (!ignore.test(i.name)) {
				current.push({
					name: i.name,
					source: i.source || "",
					dist: i.dist || "",
					type: i.type
				});
			}
		}
	}
	async output() {
		fs.ensureDirSync(this.options.dist);
		for (let viewerFile of fs
			.readdirSync(VIEWER_DIR)
			.filter(file => file.startsWith("diff"))) {
			const sourceFile = path.join(VIEWER_DIR, viewerFile);
			if (path.extname(viewerFile) === ".html") {
				const template = fs.readFileSync(sourceFile, "utf-8");
				for (let [id, items] of this.results.entries()) {
					const data: TDiffStats = {
						root: id,
						data: items
					};
					const content = template.replace(
						DIFF_STATS_PLACEHOLDER,
						JSON.stringify(data)
					);
					const casename = path.basename(id);
					const extname = path.extname(viewerFile);
					const filename = path.basename(viewerFile, extname);
					fs.writeFileSync(
						path.join(this.options.dist, `${filename}_${casename}${extname}`),
						content,
						"utf-8"
					);
				}
			} else {
				fs.copyFileSync(sourceFile, path.join(this.options.dist, viewerFile));
			}
		}
	}
}
