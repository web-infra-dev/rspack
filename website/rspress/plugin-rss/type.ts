import { Item } from 'feed';
import { PageIndexInfo } from '@rspress/shared';

export type PageFeedInfo = { href: string };

export type PageIndexInfoWithFeeds = PageIndexInfo & { feeds?: PageFeedInfo[] };
export type RssItem = Item & { feeds: string[] };
