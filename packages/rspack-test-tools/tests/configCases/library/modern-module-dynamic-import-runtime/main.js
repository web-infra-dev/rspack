import { react } from 'react' // 'module' + 'import' externalized
import { angular } from 'angular' // 'module' externalized

export const main = async () => {
	const dyn = await import('./dyn.js') // lazy dynamic import
	const reactNs = await import('react') // 'module' + 'import' externalized
  const vueNs = await import('vue') // 'import' externalized
	console.log(angular, react, reactNs, vueNs, dyn)
}
