import { isAbsolute } from "node:path";
import type {
	Configuration,
	ExternalItem,
	ExternalItemUmdValue
} from "../config";

const ERROR_PREFIX = "Invalid Rspack configuration:";

const validateContext = ({ context }: Configuration) => {
	if (context && !isAbsolute(context)) {
		throw new Error(
			`${ERROR_PREFIX} "context" must be an absolute path, get "${context}".`
		);
	}
};

const validateSplitChunks = ({ optimization }: Configuration) => {
	if (optimization?.splitChunks) {
		const { minChunks } = optimization.splitChunks;
		if (minChunks !== undefined && minChunks < 1) {
			throw new Error(
				`${ERROR_PREFIX} "optimization.splitChunks.minChunks" must be greater than or equal to 1, get \`${minChunks}\`.`
			);
		}
	}
};

const validateExternalUmd = ({
	output,
	externals,
	externalsType
}: Configuration) => {
	let isLibraryUmd = false;
	const library = output?.library;

	if (typeof library === "object" && "type" in library) {
		isLibraryUmd = library.type === "umd";
	} else {
		isLibraryUmd = output?.libraryTarget === "umd";
	}

	if (
		!isLibraryUmd ||
		(externalsType !== undefined && externalsType !== "umd")
	) {
		return;
	}

	const checkExternalItem = (externalItem: ExternalItem | undefined) => {
		if (typeof externalItem === "object" && externalItem !== null) {
			for (const value of Object.values(externalItem)) {
				checkExternalItemValue(value);
			}
		}
	};

	const checkExternalItemValue = (value: ExternalItemUmdValue | undefined) => {
		if (!value || typeof value !== "object") {
			return;
		}

		const requiredKeys = ["root", "commonjs", "commonjs2", "amd"] as const;
		if (requiredKeys.some(key => value[key] === undefined)) {
			throw new Error(
				`${ERROR_PREFIX} External object must have "root", "commonjs", "commonjs2", "amd" properties when "libraryType" or "externalsType" is "umd", get: ${JSON.stringify(
					value,
					null,
					2
				)}.`
			);
		}
	};

	if (!Array.isArray(externals)) {
		checkExternalItem(externals);
	} else {
		externals.forEach(external => checkExternalItem(external));
	}
};

/**
 * Performs configuration validation that cannot be covered by TypeScript types.
 */
export function validateRspackConfig(config: Configuration) {
	validateContext(config);
	validateSplitChunks(config);
	validateExternalUmd(config);
}
