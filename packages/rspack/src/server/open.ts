export async function openBrowser(url: string): Promise<boolean> {
	const open = (await import("open")).default;
	try {
		open(url, {});
	} catch (err) {
		return false;
	}
	return true;
}
