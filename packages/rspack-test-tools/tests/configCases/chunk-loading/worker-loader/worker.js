onmessage = async function() {
	const fetch = await import("./worker-async")
	postMessage(`Hello ${fetch.message}`)
}
