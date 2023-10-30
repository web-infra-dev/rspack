import { ECompilerType, ITestContext, TCompilerStats } from "../type";

export async function runBuild<T extends ECompilerType>(
	context: ITestContext,
	name?: string
): Promise<TCompilerStats<T> | null> {
	let stats: TCompilerStats<T> | null = null;
	await context.build<T>(
		compiler =>
			new Promise<void>((resolve, reject) => {
				compiler.run((error, stats) => {
					if (error) return reject(error);
					stats = stats;
					resolve();
				});
			}),
		name
	);
	return stats;
}
