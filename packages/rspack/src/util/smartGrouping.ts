/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

type GroupOptions = {
	groupChildren?: boolean | undefined;
	force?: boolean | undefined;
	targetGroupCount?: number | undefined;
};

export type GroupConfig = {
	getKeys: (arg0: any) => string[] | undefined;
	createGroup: <T, R>(arg0: string, arg1: (T | R)[], arg2: T[]) => R;
	getOptions?: (<T>(arg0: string, arg1: T[]) => GroupOptions) | undefined;
};

type Group<T, R> = {
	config: GroupConfig;
	name: string;
	alreadyGrouped: boolean;
	items: Set<ItemWithGroups<T, R>> | undefined;
};

type ItemWithGroups<T, R> = {
	item: T;
	groups: Set<Group<T, R>>;
};

export const smartGrouping = <T, R>(
	items: T[],
	groupConfigs: GroupConfig[]
): (T | R)[] => {
	const itemsWithGroups: Set<ItemWithGroups<T, R>> = new Set();
	const allGroups: Map<string, Group<T, R>> = new Map();
	for (const item of items) {
		const groups: Set<Group<T, R>> = new Set();
		for (let i = 0; i < groupConfigs.length; i++) {
			const groupConfig = groupConfigs[i];
			const keys = groupConfig.getKeys(item);
			if (keys) {
				for (const name of keys) {
					const key = `${i}:${name}`;
					let group = allGroups.get(key);
					if (group === undefined) {
						allGroups.set(
							key,
							(group = {
								config: groupConfig,
								name,
								alreadyGrouped: false,
								items: undefined
							})
						);
					}
					groups.add(group);
				}
			}
		}
		itemsWithGroups.add({
			item,
			groups
		});
	}

	const runGrouping = (
		itemsWithGroups: Set<ItemWithGroups<T, R>>
	): (T | R)[] => {
		const totalSize = itemsWithGroups.size;
		for (const entry of itemsWithGroups) {
			for (const group of entry.groups) {
				if (group.alreadyGrouped) continue;
				const items = group.items;
				if (items === undefined) {
					group.items = new Set([entry]);
				} else {
					items.add(entry);
				}
			}
		}

		const groupMap: Map<
			Group<T, R>,
			{
				items: Set<ItemWithGroups<T, R>>;
				options: GroupOptions | false | undefined;
				used: boolean;
			}
		> = new Map();
		for (const group of allGroups.values()) {
			if (group.items) {
				const items = group.items;
				group.items = undefined;
				groupMap.set(group, {
					items,
					options: undefined,
					used: false
				});
			}
		}

		const results: (T | R)[] = [];
		for (;;) {
			let bestGroup: Group<T, R> | undefined = undefined;
			let bestGroupSize = -1;
			let bestGroupItems = undefined;
			let bestGroupOptions = undefined;
			for (const [group, state] of groupMap) {
				const { items, used } = state;
				let options = state.options;
				if (options === undefined) {
					const groupConfig = group.config;
					state.options = options =
						(groupConfig.getOptions &&
							groupConfig.getOptions(
								group.name,
								Array.from(items, ({ item }) => item)
							)) ||
						false;
				}

				const force = options && options.force;
				if (!force) {
					if (bestGroupOptions && bestGroupOptions.force) continue;
					if (used) continue;
					if (items.size <= 1 || totalSize - items.size <= 1) {
						continue;
					}
				}
				const targetGroupCount = (options && options.targetGroupCount) || 4;
				const sizeValue = force
					? items.size
					: Math.min(
							items.size,
							(totalSize * 2) / targetGroupCount +
								itemsWithGroups.size -
								items.size
						);
				if (
					sizeValue > bestGroupSize ||
					(force && (!bestGroupOptions || !bestGroupOptions.force))
				) {
					bestGroup = group;
					bestGroupSize = sizeValue;
					bestGroupItems = items;
					bestGroupOptions = options;
				}
			}
			if (bestGroup === undefined) {
				break;
			}
			const items = new Set(bestGroupItems);
			const options = bestGroupOptions;

			const groupChildren = !options || options.groupChildren !== false;

			for (const item of items) {
				itemsWithGroups.delete(item);
				// Remove all groups that items have from the map to not select them again
				for (const group of item.groups) {
					const state = groupMap.get(group);
					if (state !== undefined) {
						state.items.delete(item);
						if (state.items.size === 0) {
							groupMap.delete(group);
						} else {
							state.options = undefined;
							if (groupChildren) {
								state.used = true;
							}
						}
					}
				}
			}
			groupMap.delete(bestGroup);

			const key = bestGroup.name;
			const groupConfig = bestGroup.config;

			const allItems = Array.from(items, ({ item }) => item);

			bestGroup.alreadyGrouped = true;
			const children = groupChildren ? runGrouping(items) : allItems;
			bestGroup.alreadyGrouped = false;

			results.push(groupConfig.createGroup(key, children, allItems));
		}
		for (const { item } of itemsWithGroups) {
			results.push(item);
		}
		return results;
	};
	return runGrouping(itemsWithGroups);
};
