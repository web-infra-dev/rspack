export const main = async () => {
	const nonExistDep = await import('non_exist_dep') // Not externalized, should throw instead of panic.
	console.log(nonExistDep)
}
