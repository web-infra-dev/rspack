export const demo = () => 'otherDemo'

{
	const demo = () => 42
	console.log.bind(demo)
}
