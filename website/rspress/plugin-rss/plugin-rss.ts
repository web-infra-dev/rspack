import { PageIndexInfo, RspressPlugin } from '@rspress/shared';
import { Feed, FeedOptions, Item } from 'feed';
import { mkdir, writeFile } from 'node:fs/promises';
import * as PathModule from 'node:path';
import { PageIndexInfoWithFeeds, RssItem } from './type';
import {
	addLinkFeeds,
	linkFeedsToPage,
	notNullish,
	PartialPartial,
	resolveUrl,
	toArray,
} from './utils';

export type UserFeedOptions = PartialPartial<
	FeedOptions,
	'title' | 'copyright' | 'id'
>;

export type UserFeedItem<K extends string = any> = Item &
	(
		| {
				/** the feed that this page belongs to */
				feed: K;
				feeds?: never;
		  }
		| {
				feed?: never;
				/** the feed list that this page belongs to */
				feeds: K[];
		  }
	);

export interface PluginRssOption<K extends string = any> {
	/**
	 * the url prefix of articles
	 */
	routePublicPath: string;
	/**
	 * to generate the feed item options from page info. return `false` to exclude a page.
	 */
	toFeedItem: (page: PageIndexInfo) => false | UserFeedItem<K>;
	/**
	 * output dir for rss file.
	 * @default `${config.outDir || 'doc_build'}/rss/`
	 */
	outDir?: string;
	/**
	 * the url prefix of rss files
	 * @default "/rss/"
	 */
	rssPublicPath?: string;
	/**
	 * `FeedOptions` of `feed` module
	 */
	feedOptions?: UserFeedOptions;
	/**
	 * `FeedOptions` of `feed` module by feed name
	 */
	feedOptionsByName?: Partial<Record<K, UserFeedOptions>>;
}

/**
 * annotations:
 * - `frontmatter.linkFeed` and `frontmatter.linkFeeds`: for linking a rss url in a page that not included in any rss files.
 */
export function pluginRss({
	routePublicPath,
	toFeedItem,
	outDir: _outDir,
	rssPublicPath = '/rss/',
	feedOptions,
	feedOptionsByName,
}: PluginRssOption): RspressPlugin {
	/** workaround for retrieving data of pages in `afterBuild` */
	let _rssWorkaround: null | Record<string, RssItem | undefined> = null;

	/** page info to rss item */
	const getRssItem = (page: PageIndexInfoWithFeeds): undefined | RssItem => {
		const item = toFeedItem(page);
		if (!item) return;

		const feeds = toArray(item.feeds, item.feed);
		if (!feeds.length) return;

		const { link: _link, ...rest } = item;

		return { ...rest, feeds, link: resolveUrl(routePublicPath, _link) };
	};

	return {
		name: 'rspack-website/plugin-rss',

		globalUIComponents: [PathModule.resolve(__dirname, 'FeedsAnnotations.tsx')],
		beforeBuild(_, isProd) {
			if (isProd) {
				_rssWorkaround = {};
			} else {
				// skip on dev build
				_rssWorkaround = null;
			}
		},
		extendPageData(pageData: PageIndexInfoWithFeeds) {
			if (!_rssWorkaround) return;

			// rspress run `extendPageData` twice for each page - we need one only
			if (!_rssWorkaround.hasOwnProperty(pageData.id)) {
				_rssWorkaround[pageData.id] = getRssItem(pageData);
			}
			const rss = _rssWorkaround[pageData.id];
			if (rss) {
				addLinkFeeds(pageData, rss.feeds);
			}
			linkFeedsToPage(rssPublicPath, pageData);
		},
		afterBuild: async function (config) {
			if (_rssWorkaround) {
				const items = Object.values(_rssWorkaround).filter(notNullish);
				const feeds: Record<string, Feed> = Object.create(null);

				for (const { feeds: feedNames, ...item } of items) {
					for (const name of feedNames) {
						feeds[name] =
							feeds[name] ||
							new Feed({
								id: name,
								title: config.title!,
								copyright: config.themeConfig?.footer?.message || '',
								description: config.description,
								...feedOptions,
								...feedOptionsByName?.[name],
							});
						feeds[name].addItem(item);
					}
				}

				const outDir = PathModule.resolve(
					_outDir || `${config.outDir || 'doc_build'}/rss/`,
				);
				for (const [file, feed] of Object.entries(feeds)) {
					await mkdir(outDir, { recursive: true });
					await writeFile(
						PathModule.resolve(outDir, `${file}.rss`),
						// better compatible than atom1
						feed.rss2(),
					);
				}
			}
			_rssWorkaround = null;
		},
	};
}
