import { RspackHotStepProcessor } from "../processor/hot-step";
import { ECompilerType, TCompilerOptions } from "../type";
import { BasicCaseCreator } from "../test/creator";
import { HotStepRunnerFactory } from "../runner";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const creators: Map<
	TTarget,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

function getCreator(target: TTarget) {
	if (!creators.has(target)) {
		creators.set(
			target,
			new BasicCaseCreator({
				clean: true,
				describe: false,
				target,
				steps: ({ name, target }) => [
					new RspackHotStepProcessor({
						name,
						target: target as TTarget
					})
				],
				runner: HotStepRunnerFactory
			})
		);
	}
	return creators.get(target)!;
}

export function createHotStepCase(
	name: string,
	src: string,
	dist: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"]
) {
	const creator = getCreator(target);
	creator.create(name, src, dist);
}
