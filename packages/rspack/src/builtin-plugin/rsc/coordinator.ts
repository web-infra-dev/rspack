import { JsCoordinator } from "@rspack/binding";
import { Compiler, GET_COMPILER_ID } from "../../Compiler";

export class Coordinator {
	#serverCompiler?: Compiler;
	#clientCompiler?: Compiler;

	#binding?: JsCoordinator;

	applyServerCompiler(compiler: Compiler) {
		this.#serverCompiler = compiler;
	}

	applyClientCompiler(compiler: Compiler) {
		this.#clientCompiler = compiler;
	}

	getBinding() {
		if (!this.#binding) {
			this.#binding = new JsCoordinator(
				() => {
					if (!this.#serverCompiler) {
						throw new Error("server compiler 没有绑定");
					}
					// TODO:
					if (this.#serverCompiler.running) {
						return;
					}
					this.#serverCompiler.watching?.invalidate();
				},
				() => {
					if (!this.#clientCompiler) {
						throw new Error("client compiler 没有绑定");
					}
					// TODO:
					if (this.#clientCompiler.running) {
						return;
					}
					this.#clientCompiler.watching?.invalidate();
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
