import { Compilation } from "../Compilation";
/**
 * This plugin adds addInclude with dependencies for all remote container references.
 * It allows container references to be available from any chunk.
 */
import type { Compiler } from "../Compiler";
import { type EntryOptions, EntryPlugin } from "../builtin-plugin";

export interface HoistContainerReferencesPluginOptions {
	/**
	 * Container name
	 */
	containerName: string | undefined;
	/**
	 * Runtime used for loading the remotes
	 */
	runtime?: string | boolean | undefined;
}

export class HoistContainerReferencesPlugin {
	private _options: HoistContainerReferencesPluginOptions;

	constructor(options: HoistContainerReferencesPluginOptions) {
		this._options = options;
	}

	/**
	 * Apply the plugin
	 * @param {Compiler} compiler the compiler instance
	 * @returns {void}
	 */
	apply(compiler: Compiler): void {
		const { containerName, runtime = undefined } = this._options;

		if (!containerName) {
			return; // Skip if no container name is provided
		}

		compiler.hooks.make.tap("HoistContainerReferencesPlugin", compilation => {
			compilation.hooks.processAssets.tapAsync(
				{
					name: "HoistContainerReferencesPlugin",
					stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONS
				},
				(_, callback) => {
					// Get all container references in the compilation
					// For each container reference, use addInclude to ensure the reference
					// is available from any chunk
					const entries = Array.from(compilation.entries.keys())
						.filter(name =>
							name.startsWith(`container-reference-${containerName}:`)
						)
						.map(name => {
							const parts = name.split(":");
							if (parts.length < 2) return null;
							return parts[1];
						})
						.filter(Boolean) as string[];

					// If no entries were found, skip
					if (entries.length === 0) {
						return callback();
					}

					let processedCount = 0;
					const errors: Error[] = [];

					// Add all container references as includes
					for (const remote of entries) {
						// Create the entry point for this remote
						const request = `container-reference-${containerName}:${remote}`;

						// Create dependency
						const entryDependency = EntryPlugin.createDependency(request);

						// Create entry options
						const entryOptions: EntryOptions = {
							name: undefined,
							runtime: typeof runtime === "string" ? runtime : undefined,
							// Make sure these entries are not affected by the splitChunks
							filename: compilation.outputOptions.chunkFilename
						};

						compilation.addInclude("", entryDependency, entryOptions, err => {
							processedCount++;
							if (err) {
								errors.push(err);
							}

							if (processedCount === entries.length) {
								if (errors.length > 0) {
									callback(errors[0]);
								} else {
									callback();
								}
							}
						});
					}
				}
			);
		});
	}
}
