import { JsModule } from "@rspack/binding";

export interface NormalizedJsModule extends JsModule {
	identifier: () => string;
}

export function normalizeJsModule(m: JsModule): NormalizedJsModule {
	return {
		...m,
		identifier: () => m.moduleIdentifier
	};
}
