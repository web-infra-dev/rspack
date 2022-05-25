import { Callback, IBundlerBase, WebpackStats } from '@speedy-js/speedy-types';
export class RspackBundler implements IBundlerBase {
	async build(): Promise<void> {}
	async reBuild(paths: string[]): Promise<void> {}
	close(callack?: Callback): void {
		console.log('not implmented');
	}
	getStats(): WebpackStats {
		return {} as any;
	}
	setStats(stats: WebpackStats): void {
		console.log('setStats');
	}
	shouldRebuild(paths: string[]): boolean {
		return true;
	}
	canRebuild(): boolean {
		return true;
	}
}
