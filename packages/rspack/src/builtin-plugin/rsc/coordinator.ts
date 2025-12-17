import { JsCoordinator } from "@rspack/binding";
import { Compiler, GET_COMPILER_ID } from "../../Compiler";

export class Coordinator {
	#serverCompiler?: Compiler;
	#clientCompiler?: Compiler;

	// 记录 server compiler 和 client compiler 编译第几次
	// 以此进行 server compiler 和 client compiler 编译状态的同步
	#serverCompilerCount = 0;
	#clientCompilerCount = 0;

	#binding?: JsCoordinator;

	applyServerCompiler(compiler: Compiler) {
		console.log("applyServerCompiler");
		this.#serverCompiler = compiler;
		this.#serverCompiler.hooks.beforeRun.tap("RSCPlugin", () => {
			console.log("server compiler beforeRun");
			this.#serverCompilerCount++;
			if (this.#clientCompilerCount !== this.#serverCompilerCount) {
				this.#clientCompiler!.watching?.invalidate();
			}
		});
	}

	applyClientCompiler(compiler: Compiler) {
		console.log("applyClientCompiler");
		this.#clientCompiler = compiler;
		this.#clientCompiler.hooks.beforeRun.tap("RSCPlugin", () => {
			console.log("client compiler beforeRun");
			this.#clientCompilerCount++;
			if (this.#clientCompilerCount !== this.#serverCompilerCount) {
				this.#serverCompiler!.watching?.invalidate();
			}
		});
	}

	getBinding() {
		if (!this.#binding) {
			this.#binding = new JsCoordinator(
				// invalidateServerCompilerJsFn
				() => {
					// if (!this.#serverCompiler) {
					// 	throw new Error("server compiler 没有绑定");
					// }
					// // TODO:
					// if (this.#serverCompiler.running) {
					// 	return;
					// }
					// this.#serverCompiler.watching?.invalidate();
				},
				() => {
					// if (!this.#clientCompiler) {
					// 	throw new Error("client compiler 没有绑定");
					// }
					// // TODO:
					// if (this.#clientCompiler.running) {
					// 	return;
					// }
					// this.#clientCompiler.watching?.invalidate();
				},
				() => {
					if (!this.#serverCompiler) {
						throw new Error("server compiler 没有绑定");
					}
					return this.#serverCompiler[GET_COMPILER_ID]();
				}
			);
		}
		return this.#binding;
	}
}
