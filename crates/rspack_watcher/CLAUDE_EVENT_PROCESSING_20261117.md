# Watcher event processing

## Responsibilities

`Trigger` queues disk and scan events without blocking the OS watcher thread.
The event processor resolves aliases, maintains observed state, and delivers
changes to logical subscriptions. `WatchPlan` merges the physical watches needed
by those subscriptions.

Subscription selection may await a JavaScript `ignored` callback before observing
the final filesystem state. An empty selection must not skip state maintenance
or recovery of independently registered descendants. After recovery, only
nonempty change sets are delivered to the executor.

## Invariants

1. **State maintenance is independent of notification eligibility.** A removed
   ancestor invalidates its recorded descendants even when the ancestor has no
   subscriber. An appearing ancestor can recover registered descendants without
   producing a notification for its own ignored entry.
2. **Logical subscriptions are additive.** An explicit file dependency and a
   parent context can refer to the same directory link. Neither replaces the
   other. Dependency purpose does not determine the target's filesystem type;
   physical watch deduplication must preserve every logical subscription.
3. **Observed absence invalidates the old mtime baseline.** A path restored after
   an observed removal cannot be suppressed using its pre-removal timestamp.
   A replacement whose absence was never observed retains ordinary mtime
   filtering.
4. **Link identity includes its destination.** Equal link mtimes do not prove
   equal destinations. Compare the stored link destination on relevant link
   events; do not read contents or resolve unrelated paths for this check.
5. **Context scans retain their subscription and filtering decision.** Delivery
   must not rediscover the owner or repeat its `ignored` check, including when
   link following is disabled.

`followSymlinks` adds link-to-target relationships. Events without those
relationships share the same state and subscription rules in both modes.

## Filtering and cost

Keep the logical dependency path, resolved target, and reported event path
distinct. Evaluate `ignored` according to the subscription being considered;
rejecting a context entry must not reject an independent file dependency.
Only accepted subscriptions can publish their corresponding time information
and notifications.

Invalidate records using the existing descendant index. Reuse `WatchPlan` to
merge OS watches, and retain the per-refresh `LinkResolver` cache. Do not add
whole-tree canonicalization, file content hashing, or cross-refresh resolution
caching to event handling.

## Regression coverage

`native-watcher-event-invariants.mjs` exercises ignored ancestors with independent
missing dependencies, overlapping directory-link subscriptions, observed ancestor
removal followed by equal-mtime recovery, and equal-mtime link retargeting.
Checks cover notifications and intermediate timestamp state. Watchpack controls
cover ignored ancestors and overlapping subscriptions. Equal-mtime link
retargeting intentionally preserves the native watcher's previous capability;
Watchpack 2.6.3 also misses that change.
