import { getOtp } from "@continuous-auth/client";

async function main() {
	const otp1 = await getOtp("test");
	console.log("the first", otp1);
	console.log(process.env.NPM_TOKEN.slice(0, 10));

	await new Promise(resolve => setTimeout(resolve, 35000));
	const otp2 = await getOtp("test");
	console.log("the first", otp2);

	console.log(process.env.NPM_TOKEN.slice(0, 10));
}

main().then(
	() => {
		console.log("done");
		process.exit(0);
	},
	err => {
		console.log(err);
		process.exit(1);
	}
);
