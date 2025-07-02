// @ts-check
/**
 * @param {import("@octokit/rest")} github
 */
module.exports = async function action({ github, context }) {
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
			baseCommit = commit;
			const data = await fetchDataBySha(commit.sha);
			if (data?.size) {
				baseSize = data.size;
				console.log(`Commit ${commit.sha} has binary size: ${data.size}`);
				break;
			}
		} catch (e) {
			console.log(e);
		}
	}

	if (!baseCommit) {
		console.log(
			`No base binary size found within ${commits.data.length} commits`
		);
		return;
	}

	await commentToPullRequest(github, context, `testing ${baseSize}`);
};

async function commentToPullRequest(github, context, comment) {
	console.log(context);

	await github.rest.issues.createComment({
		owner: context.repo.owner,
		repo: context.repo.repo,
		issue_number: context.payload.number,
		body: `${SIZE_LIMIT_HEADING}\n Base binary size: ${comment} bytes`
	});
}

function fetchDataBySha(sha) {
	console.log(
		`trying ${DATA_URL_BASE}/commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`
	);
	return fetch(
		`${DATA_URL_BASE}/commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`
	).then(res => res.json());
}

const SIZE_LIMIT_HEADING = "## 📦 Binary Size-limit";

const DATA_URL_BASE =
	"https://raw.githubusercontent.com/web-infra-dev/rspack-ecosystem-benchmark/data";
