import { PageIndexInfoWithFeeds } from './type';
import { resolve as _resolveUrl } from 'node:url';
import { FeedOptions } from 'feed';

export type PartialPartial<T, K extends keyof T> = Partial<Pick<T, K>> &
	Omit<T, K>;

export function notNullish<T>(n: T | undefined | null): n is T {
	return n !== undefined && n !== null;
}
export function toArray<T>(arr: T[] | undefined, single: T | undefined) {
	return ([] as (T | undefined)[])
		.concat(arr || [])
		.concat([single])
		.filter(notNullish);
}

/** extends frontmatter.linkFeeds with feeds list */
export function addLinkFeeds(page: PageIndexInfoWithFeeds, feeds: string[]) {
	page.frontmatter.linkFeeds = page.frontmatter.linkFeeds || [];
	(page.frontmatter.linkFeeds as string[]).push(...feeds);
}

/** map all linked feeds to full url into `page.feeds` */
export function linkFeedsToPage(
	publicPath: string,
	page: PageIndexInfoWithFeeds,
) {
	const feeds = toArray(
		page.frontmatter.linkFeeds as string[],
		page.frontmatter.linkFeed as string,
	).map((name) => ({ href: resolveUrl(publicPath, `${name}.rss`) }));

	page.feeds = (page.feeds || []).concat(feeds);
}

export function resolveUrl(from: string, ...tos: string[]) {
	return tos.reduce((url, to) => _resolveUrl(url, to), from);
}
