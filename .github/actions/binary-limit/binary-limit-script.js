const fs = require("node:fs");

const SIZE_LIMIT_HEADING = "## 📦 Binary Size-limit";
const DEFAULT_PLATFORM = "x86_64-unknown-linux-gnu";

const BINARY_PATHS = {
	"x86_64-unknown-linux-gnu": "rspack.linux-x64-gnu.node",
	"aarch64-apple-darwin": "rspack.darwin-arm64.node",
	"x86_64-pc-windows-msvc": "rspack.win32-x64-msvc.node"
};

const PLATFORM_LABELS = {
	"x86_64-unknown-linux-gnu": "Linux x64 (glibc)",
	"aarch64-apple-darwin": "macOS arm64",
	"x86_64-pc-windows-msvc": "Windows x64"
};

/**
 * @param {{ github: import('@octokit/rest'), context: any, limit: number, platform?: string }} options
 */
async function run({ github, context, limit, platform }) {
	const target = platform || DEFAULT_PLATFORM;
	const commits = await github.rest.repos.listCommits({
		owner: context.repo.owner,
		repo: context.repo.repo,
		per_page: 30
	});

	let baseSize = 0;
	let baseCommit = null;

	for (const commit of commits.data) {
		console.log(commit.sha);
		try {
			const data = await fetchDataBySha(commit.sha);
			const size = data?.sizes?.[target] ?? data?.size;
			if (typeof size === "number") {
				baseCommit = commit;
				baseSize = size;
				console.log(
					`Commit ${commit.sha} has binary size (${target}): ${size}`
				);
				break;
			}
		} catch (e) {
			console.log(e);
		}
	}

	if (!baseCommit) {
		const error = `No base binary size found within ${commits.data.length} commits`;
		console.log(error);
		throw new Error(error);
	}

	const file = getBinaryPath(target);
	console.log(`Checking binary size for ${file}`);
	const headSize = fs.statSync(file).size;

	console.log(`Base commit size (${target}): ${baseSize}`);
	console.log(`Head commit size (${target}): ${headSize}`);

	const increasedSize = headSize - baseSize;
	return {
		platform: target,
		baseSize,
		headSize,
		increasedSize,
		exceeded: increasedSize > limit,
		comment: compareBinarySize(headSize, baseSize, context, baseCommit)
	};
}

module.exports = run;
module.exports.commentToPullRequest = commentToPullRequest;
module.exports.formatReport = formatReport;
module.exports.getBinaryPath = getBinaryPath;
module.exports.SIZE_LIMIT_HEADING = SIZE_LIMIT_HEADING;

async function commentToPullRequest(github, context, body) {
	const { data: comments } = await github.rest.issues.listComments({
		owner: context.repo.owner,
		repo: context.repo.repo,
		issue_number: context.payload.number
	});

	const prevComment = comments.find(
		comment =>
			comment.user.login === "github-actions[bot]" &&
			comment.body.startsWith(SIZE_LIMIT_HEADING)
	);

	if (prevComment) {
		await github.rest.issues.updateComment({
			owner: context.repo.owner,
			repo: context.repo.repo,
			comment_id: prevComment.id,
			body
		});
		return;
	}

	await github.rest.issues.createComment({
		owner: context.repo.owner,
		repo: context.repo.repo,
		issue_number: context.payload.number,
		body
	});
}

function formatReport(entries) {
	const ordered = [...entries].sort((a, b) =>
		a.platform.localeCompare(b.platform)
	);
	const sections = ordered.map(entry => {
		const title = PLATFORM_LABELS[entry.platform] || entry.platform;
		return `### ${title}\n${entry.comment}`;
	});
	return `${SIZE_LIMIT_HEADING}\n\n${sections.join("\n\n")}`.trim();
}

function fetchDataBySha(sha) {
	const dataUrl = `${DATA_URL_BASE}/commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`;
	console.log("fetching", dataUrl, "...");
	return fetch(dataUrl).then(res => res.json());
}

const DATA_URL_BASE =
	"https://raw.githubusercontent.com/web-infra-dev/rspack-ecosystem-benchmark/data";

function compareBinarySize(headSize, baseSize, context, baseCommit) {
	const message = baseCommit.commit.message.split("\n")[0];
	const author = baseCommit.commit.author.name;
	const headSha = context.payload.pull_request?.head.sha || context.sha;

	const info = `> Comparing [\`${headSha.slice(0, 7)}\`](${context.payload.repository.html_url}/commit/${headSha}) to  [${message} by ${author}](${baseCommit.html_url})\n\n`;

	const diff = headSize - baseSize;
	const percentage = (Math.abs(diff / baseSize) * 100).toFixed(2);
	if (diff > 0) {
		return `${info}❌ Size increased by ${toHumanReadable(diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬆️${percentage}%)`;
	}
	if (diff < 0) {
		return `${info}🎉 Size decreased by ${toHumanReadable(-diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬇️${percentage}%)`;
	}
	return `${info}🙈 Size remains the same at ${toHumanReadable(headSize)}`;
}

function getBinaryPath(platform) {
	const target = platform || DEFAULT_PLATFORM;
	const filename = BINARY_PATHS[target];
	if (!filename) {
		throw new Error(`Unsupported platform: ${target}`);
	}
	return `./crates/node_binding/${filename}`;
}

function toHumanReadable(size) {
	if (size < 1024) {
		return `${size}bytes`;
	}
	if (size < 1024 * 1024) {
		return `${(size / 1024).toFixed(2)}KB`;
	}
	return `${(size / 1024 / 1024).toFixed(2)}MB`;
}
