import path from "node:path";
import { findWorkspacePackagesNoCheck } from "@pnpm/find-workspace-packages";
import semver from "semver";

async function getCommitId() {
	const result = await $`git rev-parse --short HEAD`;
	return result.stdout.replace("\n", "");
}
export async function getLastVersion(root) {
	const pkgPath = path.resolve(root, "./packages/rspack/package.json");
	const result = await import(pkgPath, {
		assert: {
			type: "json"
		}
	});
	return result.default.version;
}

export function getNextName(name) {
	if (["monorepo"].includes(name)) {
		return name;
	}
	if (name === "create-rspack") {
		return "create-rspack-canary";
	}
	const nextName = name.replace(/^@rspack/, "@rspack-canary");
	return nextName;
}

export async function getSnapshotVersion(lastVersion) {
	const commitId = await getCommitId();
	const dateTime = new Date()
		.toISOString()
		.replace(/\.\d{3}Z$/, "")
		.replace(/[^\d]/g, "");
	return `${lastVersion}-canary-${commitId}-${dateTime}`;
}
export async function version_handler(version, options) {
	const allowedVersion = ["major", "minor", "patch", "snapshot"];
	const allowPretags = ["alpha", "beta", "rc"];
	const { pre } = options;
	if (!allowedVersion.includes(version)) {
		throw new Error(
			`version must be one of ${allowedVersion}, but you passed ${version}`
		);
	}

	const hasPre = pre && pre !== "none";

	if (hasPre && !allowPretags.includes(pre)) {
		throw new Error(
			`pre tag must be one of ${allowPretags}, but you passed ${pre}`
		);
	}
	const root = process.cwd();

	const lastVersion = await getLastVersion(root);
	let nextVersion;
	if (version === "snapshot") {
		nextVersion = await getSnapshotVersion(semver.inc(lastVersion, "patch"));
	} else {
		if (hasPre) {
			const existsPreTag = allowPretags.find(i => lastVersion.includes(i));
			if (existsPreTag) {
				// has pre tag
				if (existsPreTag === pre) {
					// same pre tag
					nextVersion = semver.inc(lastVersion, "prerelease", pre);
				} else {
					// different pre tag
					nextVersion = `${lastVersion.split(existsPreTag)[0]}${pre}.0`;
				}
			} else {
				nextVersion = semver.inc(lastVersion, `pre${version}`, pre);
			}
		} else {
			nextVersion = semver.inc(lastVersion, version);
		}
	}
	const workspaces = await findWorkspacePackagesNoCheck(root);
	for (const workspace of workspaces) {
		// skip all example upgrade
		if (
			workspace.manifest.name?.includes("example-") ||
			(workspace.manifest.private === true &&
				workspace.manifest.name !== "monorepo")
		) {
			continue;
		}
		let newManifest;

		if (version === "snapshot") {
			const nextName = getNextName(workspace.manifest.name);
			newManifest = {
				...workspace.manifest,
				name: nextName,
				version: nextVersion
			};
		} else {
			newManifest = {
				...workspace.manifest,
				version: nextVersion
			};
		}
		console.log(newManifest.name);
		workspace.writeProjectManifest(newManifest);
	}
}
