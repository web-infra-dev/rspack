import { lit } from 'lit' // 'module' + 'import' externalized
import { svelte } from 'svelte' // 'module' externalized

export default dynamic = async () => {
  const litNs = await import('lit') // 'module' + 'import' externalized
  const solidNs = await import('solid') // 'import' externalized
	console.log(svelte, lit, litNs, solidNs)
}

