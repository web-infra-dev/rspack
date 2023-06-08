!(function () {
	var e = {
			"../../node_modules/rxjs/dist/esm5/index.js": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					Observable: function () {
						return r.Observable;
					},
					ConnectableObservable: function () {
						return o.ConnectableObservable;
					},
					Subject: function () {
						return i.Subject;
					},
					BehaviorSubject: function () {
						return s.BehaviorSubject;
					},
					Subscription: function () {
						return l.Subscription;
					},
					pipe: function () {
						return a.pipe;
					},
					isObservable: function () {
						return u.isObservable;
					},
					EmptyError: function () {
						return d.EmptyError;
					},
					combineLatest: function () {
						return c.combineLatest;
					},
					concat: function () {
						return f.concat;
					},
					defer: function () {
						return h.defer;
					},
					from: function () {
						return m.from;
					},
					merge: function () {
						return g.merge;
					},
					of: function () {
						return v.of;
					},
					throwError: function () {
						return y.throwError;
					},
					EMPTY: function () {
						return p.EMPTY;
					},
				});
				var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/ConnectableObservable.js",
					);
				n("../../node_modules/rxjs/dist/esm5/internal/symbol/observable.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/dom/animationFrames.js",
					);
				var i = n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
					s = n(
						"../../node_modules/rxjs/dist/esm5/internal/BehaviorSubject.js",
					);
				n("../../node_modules/rxjs/dist/esm5/internal/ReplaySubject.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/AsyncSubject.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/scheduler/asap.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/scheduler/queue.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/animationFrame.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/VirtualTimeScheduler.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/Scheduler.js");
				var l = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js");
				n("../../node_modules/rxjs/dist/esm5/internal/Subscriber.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/Notification.js");
				var a = n("../../node_modules/rxjs/dist/esm5/internal/util/pipe.js");
				n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js");
				var u = n(
					"../../node_modules/rxjs/dist/esm5/internal/util/isObservable.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/lastValueFrom.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/firstValueFrom.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/ArgumentOutOfRangeError.js",
					);
				var d = n(
					"../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/util/NotFoundError.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/ObjectUnsubscribedError.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/util/SequenceError.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/timeout.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/UnsubscriptionError.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/bindCallback.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/bindNodeCallback.js",
					);
				var c = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/combineLatest.js",
					),
					f = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/concat.js",
					);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/connectable.js",
				);
				var h = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/defer.js",
					),
					p = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js",
					);
				n("../../node_modules/rxjs/dist/esm5/internal/observable/forkJoin.js");
				var m = n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/from.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/observable/fromEvent.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/fromEventPattern.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/generate.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/iif.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/interval.js",
					);
				var g = n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/merge.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/observable/never.js");
				var v = n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/of.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/onErrorResumeNext.js",
				),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/pairs.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/partition.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/race.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/range.js");
				var y = n(
					"../../node_modules/rxjs/dist/esm5/internal/observable/throwError.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/using.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/zip.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduled.js",
					),
					n.es(n("../../node_modules/rxjs/dist/esm5/internal/types.js"), t),
					n("../../node_modules/rxjs/dist/esm5/internal/config.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/audit.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/auditTime.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/buffer.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferCount.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferToggle.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferWhen.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/catchError.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestWith.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatMap.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatMapTo.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/connect.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/count.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/debounce.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/debounceTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/delay.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/delayWhen.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/dematerialize.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/distinct.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilChanged.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilKeyChanged.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/elementAt.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/endWith.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/every.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/exhaust.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustMap.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/expand.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/filter.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/finalize.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/find.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/findIndex.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/first.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/groupBy.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/ignoreElements.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/isEmpty.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/last.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/map.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/mapTo.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/materialize.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/max.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/flatMap.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMapTo.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeScan.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/min.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/multicast.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/onErrorResumeNextWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/pairwise.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/pluck.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/publish.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishBehavior.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishLast.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishReplay.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/raceWith.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/repeat.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/repeatWhen.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/retry.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/retryWhen.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/refCount.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/sample.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/sampleTime.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/scan.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/sequenceEqual.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/share.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/shareReplay.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/single.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/skip.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/skipLast.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/skipUntil.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/skipWhile.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/startWith.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchMapTo.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchScan.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/take.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/takeLast.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/takeUntil.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/takeWhile.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/tap.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/throttle.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/throttleTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timeInterval.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timeoutWith.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timestamp.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/toArray.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/window.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowCount.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowToggle.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowWhen.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/withLatestFrom.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/zipAll.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/zipWith.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/AsyncSubject.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				var r = n("../../node_modules/tslib/tslib.es6.js");
				!(function (e) {
					function t() {
						var t = (null !== e && e.apply(this, arguments)) || this;
						return (
							(t._value = null), (t._hasValue = !1), (t._isComplete = !1), t
						);
					}
					(0, r.__extends)(t, e),
						(t.prototype._checkFinalizedStatuses = function (e) {
							var t = this.hasError,
								n = this._hasValue,
								r = this._value,
								o = this.thrownError,
								i = this.isStopped,
								s = this._isComplete;
							t ? e.error(o) : (i || s) && (n && e.next(r), e.complete());
						}),
						(t.prototype.next = function (e) {
							!this.isStopped && ((this._value = e), (this._hasValue = !0));
						}),
						(t.prototype.complete = function () {
							var t = this._hasValue,
								n = this._value;
							!this._isComplete &&
								((this._isComplete = !0),
								t && e.prototype.next.call(this, n),
								e.prototype.complete.call(this));
						});
				})(n("../../node_modules/rxjs/dist/esm5/internal/Subject.js").Subject);
			},
			"../../node_modules/rxjs/dist/esm5/internal/BehaviorSubject.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "BehaviorSubject", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t(t) {
								var n = e.call(this) || this;
								return (n._value = t), n;
							}
							return (
								(0, r.__extends)(t, e),
								Object.defineProperty(t.prototype, "value", {
									get: function () {
										return this.getValue();
									},
									enumerable: !1,
									configurable: !0,
								}),
								(t.prototype._subscribe = function (t) {
									var n = e.prototype._subscribe.call(this, t);
									return n.closed || t.next(this._value), n;
								}),
								(t.prototype.getValue = function () {
									var e = this.hasError,
										t = this.thrownError,
										n = this._value;
									if (e) throw t;
									return this._throwIfClosed(), n;
								}),
								(t.prototype.next = function (t) {
									e.prototype.next.call(this, (this._value = t));
								}),
								t
							);
						})(
							n("../../node_modules/rxjs/dist/esm5/internal/Subject.js")
								.Subject,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/Notification.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js",
					),
					o = n("../../node_modules/rxjs/dist/esm5/internal/observable/of.js"),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/throwError.js",
					),
					s = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					l =
						(((l = l || {}).NEXT = "N"),
						(l.ERROR = "E"),
						(l.COMPLETE = "C"),
						l);
				!(function () {
					function e(e, t, n) {
						(this.kind = e),
							(this.value = t),
							(this.error = n),
							(this.hasValue = "N" === e);
					}
					(e.prototype.observe = function (e) {
						return (function (e, t) {
							var n,
								r,
								o,
								i = e.kind,
								s = e.value,
								l = e.error;
							if ("string" != typeof i)
								throw TypeError('Invalid notification, missing "kind"');
							"N" === i
								? null === (n = t.next) || void 0 === n || n.call(t, s)
								: "E" === i
								? null === (r = t.error) || void 0 === r || r.call(t, l)
								: null === (o = t.complete) || void 0 === o || o.call(t);
						})(this, e);
					}),
						(e.prototype.do = function (e, t, n) {
							var r = this.kind,
								o = this.value,
								i = this.error;
							return "N" === r
								? null == e
									? void 0
									: e(o)
								: "E" === r
								? null == t
									? void 0
									: t(i)
								: null == n
								? void 0
								: n();
						}),
						(e.prototype.accept = function (e, t, n) {
							return (0, s.isFunction)(null == e ? void 0 : e.next)
								? this.observe(e)
								: this.do(e, t, n);
						}),
						(e.prototype.toObservable = function () {
							var e = this.kind,
								t = this.value,
								n = this.error,
								s =
									"N" === e
										? (0, o.of)(t)
										: "E" === e
										? (0, i.throwError)(function () {
												return n;
										  })
										: "C" === e
										? r.EMPTY
										: 0;
							if (!s) throw TypeError("Unexpected notification kind " + e);
							return s;
						}),
						(e.createNext = function (t) {
							return new e("N", t);
						}),
						(e.createError = function (t) {
							return new e("E", void 0, t);
						}),
						(e.createComplete = function () {
							return e.completeNotification;
						}),
						(e.completeNotification = new e("C"));
				})();
			},
			"../../node_modules/rxjs/dist/esm5/internal/NotificationFactories.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					!(function (e, t) {
						for (var n in t)
							Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
					})(t, {
						COMPLETE_NOTIFICATION: function () {
							return r;
						},
						errorNotification: function () {
							return o;
						},
						nextNotification: function () {
							return i;
						},
					});
					var r = s("C", void 0, void 0);
					function o(e) {
						return s("E", void 0, e);
					}
					function i(e) {
						return s("N", e, void 0);
					}
					function s(e, t, n) {
						return { kind: e, value: t, error: n };
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/Observable.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "Observable", {
						enumerable: !0,
						get: function () {
							return d;
						},
					});
				var r = n("../../node_modules/rxjs/dist/esm5/internal/Subscriber.js"),
					o = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/symbol/observable.js",
					),
					s = n("../../node_modules/rxjs/dist/esm5/internal/util/pipe.js"),
					l = n("../../node_modules/rxjs/dist/esm5/internal/config.js"),
					a = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					u = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/errorContext.js",
					),
					d = (function () {
						function e(e) {
							e && (this._subscribe = e);
						}
						return (
							(e.prototype.lift = function (t) {
								var n = new e();
								return (n.source = this), (n.operator = t), n;
							}),
							(e.prototype.subscribe = function (e, t, n) {
								var i = this,
									s = (function (e) {
										var t;
										return (
											(e && e instanceof r.Subscriber) ||
											((t = e) &&
												(0, a.isFunction)(t.next) &&
												(0, a.isFunction)(t.error) &&
												(0, a.isFunction)(t.complete) &&
												(0, o.isSubscription)(e))
										);
									})(e)
										? e
										: new r.SafeSubscriber(e, t, n);
								return (
									(0, u.errorContext)(function () {
										var e = i.operator,
											t = i.source;
										s.add(
											e
												? e.call(s, t)
												: t
												? i._subscribe(s)
												: i._trySubscribe(s),
										);
									}),
									s
								);
							}),
							(e.prototype._trySubscribe = function (e) {
								try {
									return this._subscribe(e);
								} catch (t) {
									e.error(t);
								}
							}),
							(e.prototype.forEach = function (e, t) {
								var n = this;
								return new (t = c(t))(function (t, o) {
									var i = new r.SafeSubscriber({
										next: function (t) {
											try {
												e(t);
											} catch (e) {
												o(e), i.unsubscribe();
											}
										},
										error: o,
										complete: t,
									});
									n.subscribe(i);
								});
							}),
							(e.prototype._subscribe = function (e) {
								var t;
								return null === (t = this.source) || void 0 === t
									? void 0
									: t.subscribe(e);
							}),
							(e.prototype[i.observable] = function () {
								return this;
							}),
							(e.prototype.pipe = function () {
								for (var e = [], t = 0; t < arguments.length; t++)
									e[t] = arguments[t];
								return (0, s.pipeFromArray)(e)(this);
							}),
							(e.prototype.toPromise = function (e) {
								var t = this;
								return new (e = c(e))(function (e, n) {
									var r;
									t.subscribe(
										function (e) {
											return (r = e);
										},
										function (e) {
											return n(e);
										},
										function () {
											return e(r);
										},
									);
								});
							}),
							(e.create = function (t) {
								return new e(t);
							}),
							e
						);
					})();
				function c(e) {
					var t;
					return null !== (t = null != e ? e : l.config.Promise) && void 0 !== t
						? t
						: Promise;
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/ReplaySubject.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				var r = n("../../node_modules/tslib/tslib.es6.js"),
					o = n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/dateTimestampProvider.js",
					);
				!(function (e) {
					function t(t, n, r) {
						void 0 === t && (t = 1 / 0),
							void 0 === n && (n = 1 / 0),
							void 0 === r && (r = i.dateTimestampProvider);
						var o = e.call(this) || this;
						return (
							(o._bufferSize = t),
							(o._windowTime = n),
							(o._timestampProvider = r),
							(o._buffer = []),
							(o._infiniteTimeWindow = !0),
							(o._infiniteTimeWindow = n === 1 / 0),
							(o._bufferSize = Math.max(1, t)),
							(o._windowTime = Math.max(1, n)),
							o
						);
					}
					(0, r.__extends)(t, e),
						(t.prototype.next = function (t) {
							var n = this.isStopped,
								r = this._buffer,
								o = this._infiniteTimeWindow,
								i = this._timestampProvider,
								s = this._windowTime;
							!n && (r.push(t), o || r.push(i.now() + s)),
								this._trimBuffer(),
								e.prototype.next.call(this, t);
						}),
						(t.prototype._subscribe = function (e) {
							this._throwIfClosed(), this._trimBuffer();
							for (
								var t = this._innerSubscribe(e),
									n = this._infiniteTimeWindow,
									r = this._buffer.slice(),
									o = 0;
								o < r.length && !e.closed;
								o += n ? 1 : 2
							)
								e.next(r[o]);
							return this._checkFinalizedStatuses(e), t;
						}),
						(t.prototype._trimBuffer = function () {
							var e = this._bufferSize,
								t = this._timestampProvider,
								n = this._buffer,
								r = this._infiniteTimeWindow,
								o = (r ? 1 : 2) * e;
							if (
								(e < 1 / 0 && o < n.length && n.splice(0, n.length - o), !r)
							) {
								for (
									var i = t.now(), s = 0, l = 1;
									l < n.length && n[l] <= i;
									l += 2
								)
									s = l;
								s && n.splice(0, s + 1);
							}
						});
				})(o.Subject);
			},
			"../../node_modules/rxjs/dist/esm5/internal/Scheduler.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "Scheduler", {
						enumerable: !0,
						get: function () {
							return o;
						},
					});
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/dateTimestampProvider.js",
					),
					o = (function () {
						function e(t, n) {
							void 0 === n && (n = e.now),
								(this.schedulerActionCtor = t),
								(this.now = n);
						}
						return (
							(e.prototype.schedule = function (e, t, n) {
								return (
									void 0 === t && (t = 0),
									new this.schedulerActionCtor(this, e).schedule(n, t)
								);
							}),
							(e.now = r.dateTimestampProvider.now),
							e
						);
					})();
			},
			"../../node_modules/rxjs/dist/esm5/internal/Subject.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "Subject", {
						enumerable: !0,
						get: function () {
							return u;
						},
					});
				var r = n("../../node_modules/tslib/tslib.es6.js"),
					o = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
					i = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
					s = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/ObjectUnsubscribedError.js",
					),
					l = n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js"),
					a = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/errorContext.js",
					),
					u = (function (e) {
						function t() {
							var t = e.call(this) || this;
							return (
								(t.closed = !1),
								(t.currentObservers = null),
								(t.observers = []),
								(t.isStopped = !1),
								(t.hasError = !1),
								(t.thrownError = null),
								t
							);
						}
						return (
							(0, r.__extends)(t, e),
							(t.prototype.lift = function (e) {
								var t = new d(this, this);
								return (t.operator = e), t;
							}),
							(t.prototype._throwIfClosed = function () {
								if (this.closed) throw new s.ObjectUnsubscribedError();
							}),
							(t.prototype.next = function (e) {
								var t = this;
								(0, a.errorContext)(function () {
									var n, o;
									if ((t._throwIfClosed(), !t.isStopped)) {
										!t.currentObservers &&
											(t.currentObservers = Array.from(t.observers));
										try {
											for (
												var i = (0, r.__values)(t.currentObservers),
													s = i.next();
												!s.done;
												s = i.next()
											)
												s.value.next(e);
										} catch (e) {
											n = { error: e };
										} finally {
											try {
												s && !s.done && (o = i.return) && o.call(i);
											} finally {
												if (n) throw n.error;
											}
										}
									}
								});
							}),
							(t.prototype.error = function (e) {
								var t = this;
								(0, a.errorContext)(function () {
									if ((t._throwIfClosed(), !t.isStopped)) {
										(t.hasError = t.isStopped = !0), (t.thrownError = e);
										for (var n = t.observers; n.length; ) n.shift().error(e);
									}
								});
							}),
							(t.prototype.complete = function () {
								var e = this;
								(0, a.errorContext)(function () {
									if ((e._throwIfClosed(), !e.isStopped)) {
										e.isStopped = !0;
										for (var t = e.observers; t.length; ) t.shift().complete();
									}
								});
							}),
							(t.prototype.unsubscribe = function () {
								(this.isStopped = this.closed = !0),
									(this.observers = this.currentObservers = null);
							}),
							Object.defineProperty(t.prototype, "observed", {
								get: function () {
									var e;
									return (
										(null === (e = this.observers) || void 0 === e
											? void 0
											: e.length) > 0
									);
								},
								enumerable: !1,
								configurable: !0,
							}),
							(t.prototype._trySubscribe = function (t) {
								return (
									this._throwIfClosed(), e.prototype._trySubscribe.call(this, t)
								);
							}),
							(t.prototype._subscribe = function (e) {
								return (
									this._throwIfClosed(),
									this._checkFinalizedStatuses(e),
									this._innerSubscribe(e)
								);
							}),
							(t.prototype._innerSubscribe = function (e) {
								var t = this,
									n = this.hasError,
									r = this.isStopped,
									o = this.observers;
								return n || r
									? i.EMPTY_SUBSCRIPTION
									: ((this.currentObservers = null),
									  o.push(e),
									  new i.Subscription(function () {
											(t.currentObservers = null), (0, l.arrRemove)(o, e);
									  }));
							}),
							(t.prototype._checkFinalizedStatuses = function (e) {
								var t = this.hasError,
									n = this.thrownError,
									r = this.isStopped;
								t ? e.error(n) : r && e.complete();
							}),
							(t.prototype.asObservable = function () {
								var e = new o.Observable();
								return (e.source = this), e;
							}),
							(t.create = function (e, t) {
								return new d(e, t);
							}),
							t
						);
					})(o.Observable),
					d = (function (e) {
						function t(t, n) {
							var r = e.call(this) || this;
							return (r.destination = t), (r.source = n), r;
						}
						return (
							(0, r.__extends)(t, e),
							(t.prototype.next = function (e) {
								var t, n;
								null ===
									(n =
										null === (t = this.destination) || void 0 === t
											? void 0
											: t.next) ||
									void 0 === n ||
									n.call(t, e);
							}),
							(t.prototype.error = function (e) {
								var t, n;
								null ===
									(n =
										null === (t = this.destination) || void 0 === t
											? void 0
											: t.error) ||
									void 0 === n ||
									n.call(t, e);
							}),
							(t.prototype.complete = function () {
								var e, t;
								null ===
									(t =
										null === (e = this.destination) || void 0 === e
											? void 0
											: e.complete) ||
									void 0 === t ||
									t.call(e);
							}),
							(t.prototype._subscribe = function (e) {
								var t, n;
								return null !==
									(n =
										null === (t = this.source) || void 0 === t
											? void 0
											: t.subscribe(e)) && void 0 !== n
									? n
									: i.EMPTY_SUBSCRIPTION;
							}),
							t
						);
					})(u);
			},
			"../../node_modules/rxjs/dist/esm5/internal/Subscriber.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					Subscriber: function () {
						return f;
					},
					SafeSubscriber: function () {
						return g;
					},
				});
				var r = n("../../node_modules/tslib/tslib.es6.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					i = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
					s = n("../../node_modules/rxjs/dist/esm5/internal/config.js"),
					l = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/reportUnhandledError.js",
					),
					a = n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
					u = n(
						"../../node_modules/rxjs/dist/esm5/internal/NotificationFactories.js",
					),
					d = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/timeoutProvider.js",
					),
					c = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/errorContext.js",
					),
					f = (function (e) {
						function t(t) {
							var n = e.call(this) || this;
							return (
								(n.isStopped = !1),
								t
									? ((n.destination = t), (0, i.isSubscription)(t) && t.add(n))
									: (n.destination = b),
								n
							);
						}
						return (
							(0, r.__extends)(t, e),
							(t.create = function (e, t, n) {
								return new g(e, t, n);
							}),
							(t.prototype.next = function (e) {
								this.isStopped
									? y((0, u.nextNotification)(e), this)
									: this._next(e);
							}),
							(t.prototype.error = function (e) {
								this.isStopped
									? y((0, u.errorNotification)(e), this)
									: ((this.isStopped = !0), this._error(e));
							}),
							(t.prototype.complete = function () {
								this.isStopped
									? y(u.COMPLETE_NOTIFICATION, this)
									: ((this.isStopped = !0), this._complete());
							}),
							(t.prototype.unsubscribe = function () {
								!this.closed &&
									((this.isStopped = !0),
									e.prototype.unsubscribe.call(this),
									(this.destination = null));
							}),
							(t.prototype._next = function (e) {
								this.destination.next(e);
							}),
							(t.prototype._error = function (e) {
								try {
									this.destination.error(e);
								} finally {
									this.unsubscribe();
								}
							}),
							(t.prototype._complete = function () {
								try {
									this.destination.complete();
								} finally {
									this.unsubscribe();
								}
							}),
							t
						);
					})(i.Subscription),
					h = Function.prototype.bind;
				function p(e, t) {
					return h.call(e, t);
				}
				var m = (function () {
						function e(e) {
							this.partialObserver = e;
						}
						return (
							(e.prototype.next = function (e) {
								var t = this.partialObserver;
								if (t.next)
									try {
										t.next(e);
									} catch (e) {
										v(e);
									}
							}),
							(e.prototype.error = function (e) {
								var t = this.partialObserver;
								if (t.error)
									try {
										t.error(e);
									} catch (e) {
										v(e);
									}
								else v(e);
							}),
							(e.prototype.complete = function () {
								var e = this.partialObserver;
								if (e.complete)
									try {
										e.complete();
									} catch (e) {
										v(e);
									}
							}),
							e
						);
					})(),
					g = (function (e) {
						function t(t, n, r) {
							var i,
								l,
								a = e.call(this) || this;
							return (
								(0, o.isFunction)(t) || !t
									? (i = {
											next: null != t ? t : void 0,
											error: null != n ? n : void 0,
											complete: null != r ? r : void 0,
									  })
									: a && s.config.useDeprecatedNextContext
									? (((l = Object.create(t)).unsubscribe = function () {
											return a.unsubscribe();
									  }),
									  (i = {
											next: t.next && p(t.next, l),
											error: t.error && p(t.error, l),
											complete: t.complete && p(t.complete, l),
									  }))
									: (i = t),
								(a.destination = new m(i)),
								a
							);
						}
						return (0, r.__extends)(t, e), t;
					})(f);
				function v(e) {
					s.config.useDeprecatedSynchronousErrorHandling
						? (0, c.captureError)(e)
						: (0, l.reportUnhandledError)(e);
				}
				function y(e, t) {
					var n = s.config.onStoppedNotification;
					n &&
						d.timeoutProvider.setTimeout(function () {
							return n(e, t);
						});
				}
				var b = {
					closed: !0,
					next: a.noop,
					error: function (e) {
						throw e;
					},
					complete: a.noop,
				};
			},
			"../../node_modules/rxjs/dist/esm5/internal/Subscription.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					Subscription: function () {
						return l;
					},
					EMPTY_SUBSCRIPTION: function () {
						return a;
					},
					isSubscription: function () {
						return u;
					},
				});
				var r = n("../../node_modules/tslib/tslib.es6.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/UnsubscriptionError.js",
					),
					s = n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js"),
					l = (function () {
						var e;
						function t(e) {
							(this.initialTeardown = e),
								(this.closed = !1),
								(this._parentage = null),
								(this._finalizers = null);
						}
						return (
							(t.prototype.unsubscribe = function () {
								if (!this.closed) {
									this.closed = !0;
									var e,
										t,
										n,
										s,
										l,
										a = this._parentage;
									if (a) {
										if (((this._parentage = null), Array.isArray(a)))
											try {
												for (
													var u = (0, r.__values)(a), c = u.next();
													!c.done;
													c = u.next()
												)
													c.value.remove(this);
											} catch (t) {
												e = { error: t };
											} finally {
												try {
													c && !c.done && (t = u.return) && t.call(u);
												} finally {
													if (e) throw e.error;
												}
											}
										else a.remove(this);
									}
									var f = this.initialTeardown;
									if ((0, o.isFunction)(f))
										try {
											f();
										} catch (e) {
											l = e instanceof i.UnsubscriptionError ? e.errors : [e];
										}
									var h = this._finalizers;
									if (h) {
										this._finalizers = null;
										try {
											for (
												var p = (0, r.__values)(h), m = p.next();
												!m.done;
												m = p.next()
											) {
												var g = m.value;
												try {
													d(g);
												} catch (e) {
													(l = null != l ? l : []),
														e instanceof i.UnsubscriptionError
															? (l = (0, r.__spreadArray)(
																	(0, r.__spreadArray)([], (0, r.__read)(l)),
																	(0, r.__read)(e.errors),
															  ))
															: l.push(e);
												}
											}
										} catch (e) {
											n = { error: e };
										} finally {
											try {
												m && !m.done && (s = p.return) && s.call(p);
											} finally {
												if (n) throw n.error;
											}
										}
									}
									if (l) throw new i.UnsubscriptionError(l);
								}
							}),
							(t.prototype.add = function (e) {
								var n;
								if (e && e !== this) {
									if (this.closed) d(e);
									else {
										if (e instanceof t) {
											if (e.closed || e._hasParent(this)) return;
											e._addParent(this);
										}
										(this._finalizers =
											null !== (n = this._finalizers) && void 0 !== n
												? n
												: []).push(e);
									}
								}
							}),
							(t.prototype._hasParent = function (e) {
								var t = this._parentage;
								return t === e || (Array.isArray(t) && t.includes(e));
							}),
							(t.prototype._addParent = function (e) {
								var t = this._parentage;
								this._parentage = Array.isArray(t)
									? (t.push(e), t)
									: t
									? [t, e]
									: e;
							}),
							(t.prototype._removeParent = function (e) {
								var t = this._parentage;
								t === e
									? (this._parentage = null)
									: Array.isArray(t) && (0, s.arrRemove)(t, e);
							}),
							(t.prototype.remove = function (e) {
								var n = this._finalizers;
								n && (0, s.arrRemove)(n, e),
									e instanceof t && e._removeParent(this);
							}),
							(t.EMPTY = ((e = new t()), (e.closed = !0), e)),
							t
						);
					})(),
					a = l.EMPTY;
				function u(e) {
					return (
						e instanceof l ||
						(e &&
							"closed" in e &&
							(0, o.isFunction)(e.remove) &&
							(0, o.isFunction)(e.add) &&
							(0, o.isFunction)(e.unsubscribe))
					);
				}
				function d(e) {
					(0, o.isFunction)(e) ? e() : e.unsubscribe();
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/config.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "config", {
						enumerable: !0,
						get: function () {
							return r;
						},
					});
				var r = {
					onUnhandledError: null,
					onStoppedNotification: null,
					Promise: void 0,
					useDeprecatedSynchronousErrorHandling: !1,
					useDeprecatedNextContext: !1,
				};
			},
			"../../node_modules/rxjs/dist/esm5/internal/firstValueFrom.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/Subscriber.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/lastValueFrom.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/observable/ConnectableObservable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "ConnectableObservable", {
							enumerable: !0,
							get: function () {
								return u;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						i = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/refCount.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						a = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						u = (function (e) {
							function t(t, n) {
								var r = e.call(this) || this;
								return (
									(r.source = t),
									(r.subjectFactory = n),
									(r._subject = null),
									(r._refCount = 0),
									(r._connection = null),
									(0, a.hasLift)(t) && (r.lift = t.lift),
									r
								);
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype._subscribe = function (e) {
									return this.getSubject().subscribe(e);
								}),
								(t.prototype.getSubject = function () {
									var e = this._subject;
									return (
										(!e || e.isStopped) &&
											(this._subject = this.subjectFactory()),
										this._subject
									);
								}),
								(t.prototype._teardown = function () {
									this._refCount = 0;
									var e = this._connection;
									(this._subject = this._connection = null),
										null == e || e.unsubscribe();
								}),
								(t.prototype.connect = function () {
									var e = this,
										t = this._connection;
									if (!t) {
										t = this._connection = new i.Subscription();
										var n = this.getSubject();
										t.add(
											this.source.subscribe(
												(0, l.createOperatorSubscriber)(
													n,
													void 0,
													function () {
														e._teardown(), n.complete();
													},
													function (t) {
														e._teardown(), n.error(t);
													},
													function () {
														return e._teardown();
													},
												),
											),
										),
											t.closed &&
												((this._connection = null), (t = i.Subscription.EMPTY));
									}
									return t;
								}),
								(t.prototype.refCount = function () {
									return (0, s.refCount)()(this);
								}),
								t
							);
						})(o.Observable);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/bindCallback.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/bindCallbackInternals.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/bindCallbackInternals.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isScheduler.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/AsyncSubject.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/bindNodeCallback.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/bindCallbackInternals.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/combineLatest.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "combineLatest", {
							enumerable: !0,
							get: function () {
								return f;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsArgArrayOrObject.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/from.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						),
						a = n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						u = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/createObject.js",
						),
						d = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						c = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						);
					function f() {
						for (var e = [], t = 0; t < arguments.length; t++)
							e[t] = arguments[t];
						var n = (0, a.popScheduler)(e),
							c = (0, a.popResultSelector)(e),
							f = (0, o.argsArgArrayOrObject)(e),
							p = f.args,
							m = f.keys;
						if (0 === p.length) return (0, i.from)([], n);
						var g = new r.Observable(
							(function (e, t, n) {
								return (
									void 0 === n && (n = s.identity),
									function (r) {
										h(
											t,
											function () {
												for (
													var o = e.length,
														s = Array(o),
														l = o,
														a = o,
														u = function (o) {
															h(
																t,
																function () {
																	var u = (0, i.from)(e[o], t),
																		c = !1;
																	u.subscribe(
																		(0, d.createOperatorSubscriber)(
																			r,
																			function (e) {
																				(s[o] = e),
																					!c && ((c = !0), a--),
																					!a && r.next(n(s.slice()));
																			},
																			function () {
																				!--l && r.complete();
																			},
																		),
																	);
																},
																r,
															);
														},
														c = 0;
													c < o;
													c++
												)
													u(c);
											},
											r,
										);
									}
								);
							})(
								p,
								n,
								m
									? function (e) {
											return (0, u.createObject)(m, e);
									  }
									: s.identity,
							),
						);
						return c ? g.pipe((0, l.mapOneOrManyArgs)(c)) : g;
					}
					function h(e, t, n) {
						e ? (0, c.executeSchedule)(n, e, t) : t();
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/concat.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "concat", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/concatAll.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/from.js",
						);
					function s() {
						for (var e = [], t = 0; t < arguments.length; t++)
							e[t] = arguments[t];
						return (0, r.concatAll)()((0, i.from)(e, (0, o.popScheduler)(e)));
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/connectable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/defer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/defer.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "defer", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
					function i(e) {
						return new r.Observable(function (t) {
							(0, o.innerFrom)(e()).subscribe(t);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/dom/animationFrames.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r,
						o = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/performanceTimestampProvider.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/animationFrameProvider.js",
						);
					new o.Observable(function (e) {
						var t = r || i.performanceTimestampProvider,
							n = t.now(),
							o = 0,
							l = function () {
								!e.closed &&
									(o = s.animationFrameProvider.requestAnimationFrame(function (
										i,
									) {
										o = 0;
										var s = t.now();
										e.next({ timestamp: r ? s : i, elapsed: s - n }), l();
									}));
							};
						return (
							l(),
							function () {
								o && s.animationFrameProvider.cancelAnimationFrame(o);
							}
						);
					});
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "EMPTY", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = new (n(
						"../../node_modules/rxjs/dist/esm5/internal/Observable.js",
					).Observable)(function (e) {
						return e.complete();
					});
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/forkJoin.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsArgArrayOrObject.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/createObject.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/from.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "from", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduled.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
					function i(e, t) {
						return t ? (0, r.scheduled)(e, t) : (0, o.innerFrom)(e);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/fromEvent.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isArrayLike.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/fromEventPattern.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/fromSubscribable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/generate.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isScheduler.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/defer.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleIterable.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/iif.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/defer.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "innerFrom", {
							enumerable: !0,
							get: function () {
								return m;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isArrayLike.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isPromise.js",
						),
						s = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isInteropObservable.js",
						),
						a = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isAsyncIterable.js",
						),
						u = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/throwUnobservableError.js",
						),
						d = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isIterable.js",
						),
						c = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isReadableStreamLike.js",
						),
						f = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						),
						h = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/reportUnhandledError.js",
						),
						p = n(
							"../../node_modules/rxjs/dist/esm5/internal/symbol/observable.js",
						);
					function m(e) {
						if (e instanceof s.Observable) return e;
						if (null != e) {
							if ((0, l.isInteropObservable)(e))
								return (function (e) {
									return new s.Observable(function (t) {
										var n = e[p.observable]();
										if ((0, f.isFunction)(n.subscribe)) return n.subscribe(t);
										throw TypeError(
											"Provided object does not correctly implement Symbol.observable",
										);
									});
								})(e);
							if ((0, o.isArrayLike)(e))
								return (function (e) {
									return new s.Observable(function (t) {
										for (var n = 0; n < e.length && !t.closed; n++)
											t.next(e[n]);
										t.complete();
									});
								})(e);
							if ((0, i.isPromise)(e))
								return (function (e) {
									return new s.Observable(function (t) {
										e.then(
											function (e) {
												!t.closed && (t.next(e), t.complete());
											},
											function (e) {
												return t.error(e);
											},
										).then(null, h.reportUnhandledError);
									});
								})(e);
							if ((0, a.isAsyncIterable)(e)) return g(e);
							if ((0, d.isIterable)(e))
								return (function (e) {
									return new s.Observable(function (t) {
										var n, o;
										try {
											for (
												var i = (0, r.__values)(e), s = i.next();
												!s.done;
												s = i.next()
											) {
												var l = s.value;
												if ((t.next(l), t.closed)) return;
											}
										} catch (e) {
											n = { error: e };
										} finally {
											try {
												s && !s.done && (o = i.return) && o.call(i);
											} finally {
												if (n) throw n.error;
											}
										}
										t.complete();
									});
								})(e);
							if ((0, c.isReadableStreamLike)(e))
								return (function (e) {
									return g((0, c.readableStreamLikeToAsyncGenerator)(e));
								})(e);
						}
						throw (0, u.createInvalidObservableTypeError)(e);
					}
					function g(e) {
						return new s.Observable(function (t) {
							(function (e, t) {
								var n, o, i, s;
								return (0, r.__awaiter)(this, void 0, void 0, function () {
									var l;
									return (0, r.__generator)(this, function (a) {
										switch (a.label) {
											case 0:
												a.trys.push([0, 5, 6, 11]),
													(n = (0, r.__asyncValues)(e)),
													(a.label = 1);
											case 1:
												return [4, n.next()];
											case 2:
												if ((o = a.sent()).done) return [3, 4];
												if (((l = o.value), t.next(l), t.closed)) return [2];
												a.label = 3;
											case 3:
												return [3, 1];
											case 4:
												return [3, 11];
											case 5:
												return (i = { error: a.sent() }), [3, 11];
											case 6:
												if (
													(a.trys.push([6, , 9, 10]),
													!(o && !o.done && (s = n.return)))
												)
													return [3, 8];
												return [4, s.call(n)];
											case 7:
												a.sent(), (a.label = 8);
											case 8:
												return [3, 10];
											case 9:
												if (i) throw i.error;
												return [7];
											case 10:
												return [7];
											case 11:
												return t.complete(), [2];
										}
									});
								});
							})(e, t).catch(function (e) {
								return t.error(e);
							});
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/interval.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/merge.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "merge", {
							enumerable: !0,
							get: function () {
								return a;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js",
						),
						s = n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/from.js",
						);
					function a() {
						for (var e = [], t = 0; t < arguments.length; t++)
							e[t] = arguments[t];
						var n = (0, s.popScheduler)(e),
							a = (0, s.popNumber)(e, 1 / 0);
						return e.length
							? 1 === e.length
								? (0, o.innerFrom)(e[0])
								: (0, r.mergeAll)(a)((0, l.from)(e, n))
							: i.EMPTY;
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/never.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js");
					new r.Observable(o.noop);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/of.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "of", {
						enumerable: !0,
						get: function () {
							return i;
						},
					});
				var r = n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/from.js",
					);
				function i() {
					for (var e = [], t = 0; t < arguments.length; t++)
						e[t] = arguments[t];
					var n = (0, r.popScheduler)(e);
					return (0, o.from)(e, n);
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/observable/onErrorResumeNext.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "onErrorResumeNext", {
							enumerable: !0,
							get: function () {
								return a;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						s = n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
					function a() {
						for (var e = [], t = 0; t < arguments.length; t++)
							e[t] = arguments[t];
						var n = (0, o.argsOrArgArray)(e);
						return new r.Observable(function (e) {
							var t = 0,
								r = function () {
									if (t < n.length) {
										var o = void 0;
										try {
											o = (0, l.innerFrom)(n[t++]);
										} catch (e) {
											r();
											return;
										}
										var a = new i.OperatorSubscriber(e, void 0, s.noop, s.noop);
										o.subscribe(a), a.add(r);
									} else e.complete();
								};
							r();
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/pairs.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/from.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/partition.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/not.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/filter.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/race.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/range.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/empty.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/throwError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "throwError", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e, t) {
						var n = (0, o.isFunction)(e)
								? e
								: function () {
										return e;
								  },
							i = function (e) {
								return e.error(n());
							};
						return new r.Observable(
							t
								? function (e) {
										return t.schedule(i, 0, e);
								  }
								: i,
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/timer.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isScheduler.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isDate.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/using.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/empty.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/observable/zip.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/tslib/tslib.es6.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/empty.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/util/args.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					!(function (e, t) {
						for (var n in t)
							Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
					})(t, {
						createOperatorSubscriber: function () {
							return o;
						},
						OperatorSubscriber: function () {
							return i;
						},
					});
					var r = n("../../node_modules/tslib/tslib.es6.js");
					function o(e, t, n, r, o) {
						return new i(e, t, n, r, o);
					}
					var i = (function (e) {
						function t(t, n, r, o, i, s) {
							var l = e.call(this, t) || this;
							return (
								(l.onFinalize = i),
								(l.shouldUnsubscribe = s),
								(l._next = n
									? function (e) {
											try {
												n(e);
											} catch (e) {
												t.error(e);
											}
									  }
									: e.prototype._next),
								(l._error = o
									? function (e) {
											try {
												o(e);
											} catch (e) {
												t.error(e);
											} finally {
												this.unsubscribe();
											}
									  }
									: e.prototype._error),
								(l._complete = r
									? function () {
											try {
												r();
											} catch (e) {
												t.error(e);
											} finally {
												this.unsubscribe();
											}
									  }
									: e.prototype._complete),
								l
							);
						}
						return (
							(0, r.__extends)(t, e),
							(t.prototype.unsubscribe = function () {
								var t;
								if (!this.shouldUnsubscribe || this.shouldUnsubscribe()) {
									var n = this.closed;
									e.prototype.unsubscribe.call(this),
										n ||
											null === (t = this.onFinalize) ||
											void 0 === t ||
											t.call(this);
								}
							}),
							t
						);
					})(
						n("../../node_modules/rxjs/dist/esm5/internal/Subscriber.js")
							.Subscriber,
					);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/audit.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/auditTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/audit.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/buffer.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/bufferCount.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/bufferTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/bufferToggle.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/bufferWhen.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/catchError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "catchError", {
							enumerable: !0,
							get: function () {
								return function e(t) {
									return (0, i.operate)(function (n, i) {
										var s,
											l = null,
											a = !1;
										(l = n.subscribe(
											(0, o.createOperatorSubscriber)(
												i,
												void 0,
												void 0,
												function (o) {
													(s = (0, r.innerFrom)(t(o, e(t)(n)))),
														l
															? (l.unsubscribe(), (l = null), s.subscribe(i))
															: (a = !0);
												},
											),
										)),
											a && (l.unsubscribe(), (l = null), s.subscribe(i));
									});
								};
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/combineAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestAll.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatest.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/combineLatest.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/pipe.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "combineLatestAll", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/combineLatest.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/joinAllInternals.js",
						);
					function i(e) {
						return (0, o.joinAllInternals)(r.combineLatest, e);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatest.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/concat.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/concatAll.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/from.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/concatAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "concatAll", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js",
					);
					function o() {
						return (0, r.mergeAll)(1);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/concatMap.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "concatMap", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e, t) {
						return (0, o.isFunction)(t)
							? (0, r.mergeMap)(e, t, 1)
							: (0, r.mergeMap)(e, 1);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/concatMapTo.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/concatMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/concatWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/concat.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/connect.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/fromSubscribable.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/count.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/debounce.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/debounceTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "defaultIfEmpty", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function i(e) {
						return (0, r.operate)(function (t, n) {
							var r = !1;
							t.subscribe(
								(0, o.createOperatorSubscriber)(
									n,
									function (e) {
										(r = !0), n.next(e);
									},
									function () {
										!r && n.next(e), n.complete();
									},
								),
							);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/delay.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/delayWhen.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/delayWhen.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/concat.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/take.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/ignoreElements.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/mapTo.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/dematerialize.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Notification.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/distinct.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilChanged.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilKeyChanged.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilChanged.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/elementAt.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/ArgumentOutOfRangeError.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/filter.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/take.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/endWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/concat.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/of.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/every.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/exhaust.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustAll.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "exhaustAll", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustMap.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
						);
					function i() {
						return (0, r.exhaustMap)(o.identity);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustMap.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "exhaustMap", {
							enumerable: !0,
							get: function () {
								return function e(t, n) {
									return n
										? function (i) {
												return i.pipe(
													e(function (e, i) {
														return (0, o.innerFrom)(t(e, i)).pipe(
															(0, r.map)(function (t, r) {
																return n(e, t, i, r);
															}),
														);
													}),
												);
										  }
										: (0, i.operate)(function (e, n) {
												var r = 0,
													i = null,
													l = !1;
												e.subscribe(
													(0, s.createOperatorSubscriber)(
														n,
														function (e) {
															!i &&
																((i = (0, s.createOperatorSubscriber)(
																	n,
																	void 0,
																	function () {
																		(i = null), l && n.complete();
																	},
																)),
																(0, o.innerFrom)(t(e, r++)).subscribe(i));
														},
														function () {
															(l = !0), i || n.complete();
														},
													),
												);
										  });
								};
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/map.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/expand.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeInternals.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/filter.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "filter", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function i(e, t) {
						return (0, r.operate)(function (n, r) {
							var i = 0;
							n.subscribe(
								(0, o.createOperatorSubscriber)(r, function (n) {
									return e.call(t, n, i++) && r.next(n);
								}),
							);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/finalize.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "finalize", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
					function o(e) {
						return (0, r.operate)(function (t, n) {
							try {
								t.subscribe(n);
							} finally {
								n.add(e);
							}
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/find.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					);
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/findIndex.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/find.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/first.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "first", {
							enumerable: !0,
							get: function () {
								return u;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/filter.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/take.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js",
						),
						a = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
						);
					function u(e, t) {
						var n = arguments.length >= 2;
						return function (u) {
							return u.pipe(
								e
									? (0, o.filter)(function (t, n) {
											return e(t, n, u);
									  })
									: a.identity,
								(0, i.take)(1),
								n
									? (0, s.defaultIfEmpty)(t)
									: (0, l.throwIfEmpty)(function () {
											return new r.EmptyError();
									  }),
							);
						};
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/flatMap.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/groupBy.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/ignoreElements.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/isEmpty.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/joinAllInternals.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "joinAllInternals", {
							enumerable: !0,
							get: function () {
								return a;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/pipe.js"),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/toArray.js",
						);
					function a(e, t) {
						return (0, i.pipe)(
							(0, l.toArray)(),
							(0, s.mergeMap)(function (t) {
								return e(t);
							}),
							t ? (0, o.mapOneOrManyArgs)(t) : r.identity,
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/last.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "last", {
						enumerable: !0,
						get: function () {
							return u;
						},
					});
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js",
					),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/filter.js",
					),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/takeLast.js",
					),
					s = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js",
					),
					l = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js",
					),
					a = n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js");
				function u(e, t) {
					var n = arguments.length >= 2;
					return function (u) {
						return u.pipe(
							e
								? (0, o.filter)(function (t, n) {
										return e(t, n, u);
								  })
								: a.identity,
							(0, i.takeLast)(1),
							n
								? (0, l.defaultIfEmpty)(t)
								: (0, s.throwIfEmpty)(function () {
										return new r.EmptyError();
								  }),
						);
					};
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/map.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "map", {
						enumerable: !0,
						get: function () {
							return i;
						},
					});
				var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					);
				function i(e, t) {
					return (0, r.operate)(function (n, r) {
						var i = 0;
						n.subscribe(
							(0, o.createOperatorSubscriber)(r, function (n) {
								r.next(e.call(t, n, i++));
							}),
						);
					});
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mapTo.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "mapTo", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/map.js",
					);
					function o(e) {
						return (0, r.map)(function () {
							return e;
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/materialize.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Notification.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/max.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/merge.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/from.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "mergeAll", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
						);
					function i(e) {
						return void 0 === e && (e = 1 / 0), (0, r.mergeMap)(o.identity, e);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeInternals.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "mergeInternals", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function s(e, t, n, s, l, a, u, d) {
						var c = [],
							f = 0,
							h = 0,
							p = !1,
							m = function () {
								p && !c.length && !f && t.complete();
							},
							g = function (e) {
								return f < s ? v(e) : c.push(e);
							},
							v = function (e) {
								a && t.next(e), f++;
								var d = !1;
								(0, r.innerFrom)(n(e, h++)).subscribe(
									(0, i.createOperatorSubscriber)(
										t,
										function (e) {
											null == l || l(e), a ? g(e) : t.next(e);
										},
										function () {
											d = !0;
										},
										void 0,
										function () {
											if (d)
												try {
													f--;
													for (; c.length && f < s; )
														!(function () {
															var e = c.shift();
															u
																? (0, o.executeSchedule)(t, u, function () {
																		return v(e);
																  })
																: v(e);
														})();
													m();
												} catch (e) {
													t.error(e);
												}
										},
									),
								);
							};
						return (
							e.subscribe(
								(0, i.createOperatorSubscriber)(t, g, function () {
									(p = !0), m();
								}),
							),
							function () {
								null == d || d();
							}
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "mergeMap", {
							enumerable: !0,
							get: function () {
								return function e(t, n, a) {
									return (void 0 === a && (a = 1 / 0), (0, l.isFunction)(n))
										? e(function (e, i) {
												return (0, r.map)(function (t, r) {
													return n(e, t, i, r);
												})((0, o.innerFrom)(t(e, i)));
										  }, a)
										: ("number" == typeof n && (a = n),
										  (0, i.operate)(function (e, n) {
												return (0, s.mergeInternals)(e, n, t, a);
										  }));
								};
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/map.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeInternals.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMapTo.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeScan.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/mergeInternals.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/mergeWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/merge.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/min.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/multicast.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/ConnectableObservable.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/connect.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "observeOn", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function s(e, t) {
						return (
							void 0 === t && (t = 0),
							(0, o.operate)(function (n, o) {
								n.subscribe(
									(0, i.createOperatorSubscriber)(
										o,
										function (n) {
											return (0, r.executeSchedule)(
												o,
												e,
												function () {
													return o.next(n);
												},
												t,
											);
										},
										function () {
											return (0, r.executeSchedule)(
												o,
												e,
												function () {
													return o.complete();
												},
												t,
											);
										},
										function (n) {
											return (0, r.executeSchedule)(
												o,
												e,
												function () {
													return o.error(n);
												},
												t,
											);
										},
									),
								);
							})
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/onErrorResumeNextWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/onErrorResumeNext.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/pairwise.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/partition.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/not.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/filter.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/pluck.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/map.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/publish.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/multicast.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/connect.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/publishBehavior.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/BehaviorSubject.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/ConnectableObservable.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/publishLast.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/AsyncSubject.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/ConnectableObservable.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/publishReplay.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/ReplaySubject.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/multicast.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/race.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/tslib/tslib.es6.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/raceWith.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/raceWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/race.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "reduce", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/scanInternals.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
					function i(e, t) {
						return (0, o.operate)(
							(0, r.scanInternals)(e, t, arguments.length >= 2, !1, !0),
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/refCount.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "refCount", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function i() {
						return (0, r.operate)(function (e, t) {
							var n = null;
							e._refCount++;
							var r = (0, o.createOperatorSubscriber)(
								t,
								void 0,
								void 0,
								void 0,
								function () {
									if (!e || e._refCount <= 0 || 0 < --e._refCount) {
										n = null;
										return;
									}
									var r = e._connection,
										o = n;
									(n = null),
										r && (!o || r === o) && r.unsubscribe(),
										t.unsubscribe();
								},
							);
							e.subscribe(r), !r.closed && (n = e.connect());
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/repeat.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/empty.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/repeatWhen.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/retry.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/retryWhen.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/sample.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/sampleTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/sample.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/interval.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/scan.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "scan", {
						enumerable: !0,
						get: function () {
							return i;
						},
					});
				var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/scanInternals.js",
					);
				function i(e, t) {
					return (0, r.operate)(
						(0, o.scanInternals)(e, t, arguments.length >= 2, !0),
					);
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/scanInternals.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scanInternals", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					);
					function o(e, t, n, o, i) {
						return function (s, l) {
							var a = n,
								u = t,
								d = 0;
							s.subscribe(
								(0, r.createOperatorSubscriber)(
									l,
									function (t) {
										var n = d++;
										(u = a ? e(u, t, n) : ((a = !0), t)), o && l.next(u);
									},
									i &&
										function () {
											a && l.next(u), l.complete();
										},
								),
							);
						};
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/sequenceEqual.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/share.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "share", {
							enumerable: !0,
							get: function () {
								return a;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						s = n("../../node_modules/rxjs/dist/esm5/internal/Subscriber.js"),
						l = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
					function a(e) {
						void 0 === e && (e = {});
						var t = e.connector,
							n =
								void 0 === t
									? function () {
											return new i.Subject();
									  }
									: t,
							r = e.resetOnError,
							a = void 0 === r || r,
							d = e.resetOnComplete,
							c = void 0 === d || d,
							f = e.resetOnRefCountZero,
							h = void 0 === f || f;
						return function (e) {
							var t,
								r,
								i,
								d = 0,
								f = !1,
								p = !1,
								m = function () {
									null == r || r.unsubscribe(), (r = void 0);
								},
								g = function () {
									m(), (t = i = void 0), (f = p = !1);
								},
								v = function () {
									var e = t;
									g(), null == e || e.unsubscribe();
								};
							return (0, l.operate)(function (e, l) {
								d++, !p && !f && m();
								var y = (i = null != i ? i : n());
								l.add(function () {
									0 == --d && !p && !f && (r = u(v, h));
								}),
									y.subscribe(l),
									!t &&
										d > 0 &&
										((t = new s.SafeSubscriber({
											next: function (e) {
												return y.next(e);
											},
											error: function (e) {
												(p = !0), m(), (r = u(g, a, e)), y.error(e);
											},
											complete: function () {
												(f = !0), m(), (r = u(g, c)), y.complete();
											},
										})),
										(0, o.innerFrom)(e).subscribe(t));
							})(e);
						};
					}
					function u(e, t) {
						for (var n = [], i = 2; i < arguments.length; i++)
							n[i - 2] = arguments[i];
						if (!0 === t) {
							e();
							return;
						}
						if (!1 !== t) {
							var l = new s.SafeSubscriber({
								next: function () {
									l.unsubscribe(), e();
								},
							});
							return (0, o.innerFrom)(
								t.apply(void 0, (0, r.__spreadArray)([], (0, r.__read)(n))),
							).subscribe(l);
						}
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/shareReplay.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/ReplaySubject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/share.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/single.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/SequenceError.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/NotFoundError.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/skip.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/filter.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/skipLast.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/skipUntil.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/skipWhile.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/startWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "startWith", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/concat.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
					function s() {
						for (var e = [], t = 0; t < arguments.length; t++)
							e[t] = arguments[t];
						var n = (0, o.popScheduler)(e);
						return (0, i.operate)(function (t, o) {
							(n ? (0, r.concat)(e, t, n) : (0, r.concat)(e, t)).subscribe(o);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "subscribeOn", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
					function o(e, t) {
						return (
							void 0 === t && (t = 0),
							(0, r.operate)(function (n, r) {
								r.add(
									e.schedule(function () {
										return n.subscribe(r);
									}, t),
								);
							})
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/switchAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "switchMap", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function s(e, t) {
						return (0, o.operate)(function (n, o) {
							var s = null,
								l = 0,
								a = !1,
								u = function () {
									return a && !s && o.complete();
								};
							n.subscribe(
								(0, i.createOperatorSubscriber)(
									o,
									function (n) {
										null == s || s.unsubscribe();
										var a = 0,
											d = l++;
										(0, r.innerFrom)(e(n, d)).subscribe(
											(s = (0, i.createOperatorSubscriber)(
												o,
												function (e) {
													return o.next(t ? t(n, e, d, a++) : e);
												},
												function () {
													(s = null), u();
												},
											)),
										);
									},
									function () {
										(a = !0), u();
									},
								),
							);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/switchMapTo.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/switchScan.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/take.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "take", {
						enumerable: !0,
						get: function () {
							return s;
						},
					});
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js",
					),
					o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					);
				function s(e) {
					return e <= 0
						? function () {
								return r.EMPTY;
						  }
						: (0, o.operate)(function (t, n) {
								var r = 0;
								t.subscribe(
									(0, i.createOperatorSubscriber)(n, function (t) {
										++r <= e && (n.next(t), e <= r && n.complete());
									}),
								);
						  });
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/takeLast.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "takeLast", {
							enumerable: !0,
							get: function () {
								return l;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/empty.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function l(e) {
						return e <= 0
							? function () {
									return o.EMPTY;
							  }
							: (0, i.operate)(function (t, n) {
									var o = [];
									t.subscribe(
										(0, s.createOperatorSubscriber)(
											n,
											function (t) {
												o.push(t), e < o.length && o.shift();
											},
											function () {
												var e, t;
												try {
													for (
														var i = (0, r.__values)(o), s = i.next();
														!s.done;
														s = i.next()
													) {
														var l = s.value;
														n.next(l);
													}
												} catch (t) {
													e = { error: t };
												} finally {
													try {
														s && !s.done && (t = i.return) && t.call(i);
													} finally {
														if (e) throw e.error;
													}
												}
												n.complete();
											},
											void 0,
											function () {
												o = null;
											},
										),
									);
							  });
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/takeUntil.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/takeWhile.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/tap.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "tap", {
						enumerable: !0,
						get: function () {
							return l;
						},
					});
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
					i = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					),
					s = n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js");
				function l(e, t, n) {
					var l =
						(0, r.isFunction)(e) || t || n
							? { next: e, error: t, complete: n }
							: e;
					return l
						? (0, o.operate)(function (e, t) {
								null === (n = l.subscribe) || void 0 === n || n.call(l);
								var n,
									r = !0;
								e.subscribe(
									(0, i.createOperatorSubscriber)(
										t,
										function (e) {
											var n;
											null === (n = l.next) || void 0 === n || n.call(l, e),
												t.next(e);
										},
										function () {
											var e;
											(r = !1),
												null === (e = l.complete) || void 0 === e || e.call(l),
												t.complete();
										},
										function (e) {
											var n;
											(r = !1),
												null === (n = l.error) || void 0 === n || n.call(l, e),
												t.error(e);
										},
										function () {
											var e, t;
											r &&
												(null === (e = l.unsubscribe) ||
													void 0 === e ||
													e.call(l)),
												null === (t = l.finalize) || void 0 === t || t.call(l);
										},
									),
								);
						  })
						: s.identity;
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/throttle.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/throttleTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/throttle.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/timer.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "throwIfEmpty", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
					function s(e) {
						return (
							void 0 === e && (e = l),
							(0, o.operate)(function (t, n) {
								var r = !1;
								t.subscribe(
									(0, i.createOperatorSubscriber)(
										n,
										function (e) {
											(r = !0), n.next(e);
										},
										function () {
											return r ? n.complete() : n.error(e());
										},
									),
								);
							})
						);
					}
					function l() {
						return new r.EmptyError();
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/timeInterval.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/timeout.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isDate.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
					);
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
					),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						),
						(0, r.createErrorClass)(function (e) {
							return function (t) {
								void 0 === t && (t = null),
									e(this),
									(this.message = "Timeout has occurred"),
									(this.name = "TimeoutError"),
									(this.info = t);
							};
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/timeoutWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/isDate.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/timeout.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/timestamp.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/dateTimestampProvider.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/map.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/toArray.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "toArray", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js",
						),
						o = n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						i = function (e, t) {
							return e.push(t), e;
						};
					function s() {
						return (0, o.operate)(function (e, t) {
							(0, r.reduce)(i, [])(e).subscribe(t);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/window.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/windowCount.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/windowTime.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/windowToggle.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/windowWhen.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/Subject.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/withLatestFrom.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/OperatorSubscriber.js",
						),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						n("../../node_modules/rxjs/dist/esm5/internal/util/identity.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/noop.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/util/args.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/zip.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					n("../../node_modules/tslib/tslib.es6.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/observable/zip.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/util/lift.js");
			},
			"../../node_modules/rxjs/dist/esm5/internal/operators/zipAll.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/rxjs/dist/esm5/internal/observable/zip.js"),
						n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/joinAllInternals.js",
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/operators/zipWith.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						n("../../node_modules/tslib/tslib.es6.js"),
						n("../../node_modules/rxjs/dist/esm5/internal/operators/zip.js");
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleArray.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduleArray", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js");
					function o(e, t) {
						return new r.Observable(function (n) {
							var r = 0;
							return t.schedule(function () {
								r === e.length
									? n.complete()
									: (n.next(e[r++]), !n.closed && this.schedule());
							});
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleAsyncIterable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduleAsyncIterable", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						);
					function i(e, t) {
						if (!e) throw Error("Iterable cannot be null");
						return new r.Observable(function (n) {
							(0, o.executeSchedule)(n, t, function () {
								var r = e[Symbol.asyncIterator]();
								(0, o.executeSchedule)(
									n,
									t,
									function () {
										r.next().then(function (e) {
											e.done ? n.complete() : n.next(e.value);
										});
									},
									0,
									!0,
								);
							});
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleIterable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduleIterable", {
							enumerable: !0,
							get: function () {
								return l;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/symbol/iterator.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js",
						);
					function l(e, t) {
						return new r.Observable(function (n) {
							var r;
							return (
								(0, s.executeSchedule)(n, t, function () {
									(r = e[o.iterator]()),
										(0, s.executeSchedule)(
											n,
											t,
											function () {
												var e, t, o;
												try {
													(t = (e = r.next()).value), (o = e.done);
												} catch (e) {
													n.error(e);
													return;
												}
												o ? n.complete() : n.next(t);
											},
											0,
											!0,
										);
								}),
								function () {
									return (
										(0, i.isFunction)(null == r ? void 0 : r.return) &&
										r.return()
									);
								}
							);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleObservable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduleObservable", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js",
						);
					function s(e, t) {
						return (0, r.innerFrom)(e).pipe(
							(0, i.subscribeOn)(t),
							(0, o.observeOn)(t),
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/schedulePromise.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "schedulePromise", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/observable/innerFrom.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js",
						);
					function s(e, t) {
						return (0, r.innerFrom)(e).pipe(
							(0, i.subscribeOn)(t),
							(0, o.observeOn)(t),
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleReadableStreamLike.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduleReadableStreamLike", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleAsyncIterable.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isReadableStreamLike.js",
						);
					function i(e, t) {
						return (0, r.scheduleAsyncIterable)(
							(0, o.readableStreamLikeToAsyncGenerator)(e),
							t,
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduled.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "scheduled", {
							enumerable: !0,
							get: function () {
								return g;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleObservable.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/schedulePromise.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleArray.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleIterable.js",
						),
						l = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleAsyncIterable.js",
						),
						a = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isInteropObservable.js",
						),
						u = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isPromise.js",
						),
						d = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isArrayLike.js",
						),
						c = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isIterable.js",
						),
						f = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isAsyncIterable.js",
						),
						h = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/throwUnobservableError.js",
						),
						p = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isReadableStreamLike.js",
						),
						m = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduled/scheduleReadableStreamLike.js",
						);
					function g(e, t) {
						if (null != e) {
							if ((0, a.isInteropObservable)(e))
								return (0, r.scheduleObservable)(e, t);
							if ((0, d.isArrayLike)(e)) return (0, i.scheduleArray)(e, t);
							if ((0, u.isPromise)(e)) return (0, o.schedulePromise)(e, t);
							if ((0, f.isAsyncIterable)(e))
								return (0, l.scheduleAsyncIterable)(e, t);
							if ((0, c.isIterable)(e)) return (0, s.scheduleIterable)(e, t);
							if ((0, p.isReadableStreamLike)(e))
								return (0, m.scheduleReadableStreamLike)(e, t);
						}
						throw (0, h.createInvalidObservableTypeError)(e);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/Action.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "Action", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t(t, n) {
								return e.call(this) || this;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.schedule = function (e, t) {
									return void 0 === t && (t = 0), this;
								}),
								t
							);
						})(
							n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js")
								.Subscription,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AnimationFrameAction.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AnimationFrameAction", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/animationFrameProvider.js",
						),
						s = (function (e) {
							function t(t, n) {
								var r = e.call(this, t, n) || this;
								return (r.scheduler = t), (r.work = n), r;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.requestAsyncId = function (t, n, r) {
									return (void 0 === r && (r = 0), null !== r && r > 0)
										? e.prototype.requestAsyncId.call(this, t, n, r)
										: (t.actions.push(this),
										  t._scheduled ||
												(t._scheduled =
													i.animationFrameProvider.requestAnimationFrame(
														function () {
															return t.flush(void 0);
														},
													)));
								}),
								(t.prototype.recycleAsyncId = function (t, n, r) {
									if (
										(void 0 === r && (r = 0),
										null != r ? r > 0 : this.delay > 0)
									)
										return e.prototype.recycleAsyncId.call(this, t, n, r);
									var o,
										s = t.actions;
									null != n &&
										(null === (o = s[s.length - 1]) || void 0 === o
											? void 0
											: o.id) !== n &&
										(i.animationFrameProvider.cancelAnimationFrame(n),
										(t._scheduled = void 0));
								}),
								t
							);
						})(o.AsyncAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AnimationFrameScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AnimationFrameScheduler", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t() {
								return (null !== e && e.apply(this, arguments)) || this;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.flush = function (e) {
									this._active = !0;
									var t,
										n = this._scheduled;
									this._scheduled = void 0;
									var r = this.actions;
									e = e || r.shift();
									do if ((t = e.execute(e.state, e.delay))) break;
									while ((e = r[0]) && e.id === n && r.shift());
									if (((this._active = !1), t)) {
										for (; (e = r[0]) && e.id === n && r.shift(); )
											e.unsubscribe();
										throw t;
									}
								}),
								t
							);
						})(
							n(
								"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js",
							).AsyncScheduler,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsapAction.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AsapAction", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/immediateProvider.js",
						),
						s = (function (e) {
							function t(t, n) {
								var r = e.call(this, t, n) || this;
								return (r.scheduler = t), (r.work = n), r;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.requestAsyncId = function (t, n, r) {
									return (void 0 === r && (r = 0), null !== r && r > 0)
										? e.prototype.requestAsyncId.call(this, t, n, r)
										: (t.actions.push(this),
										  t._scheduled ||
												(t._scheduled = i.immediateProvider.setImmediate(
													t.flush.bind(t, void 0),
												)));
								}),
								(t.prototype.recycleAsyncId = function (t, n, r) {
									if (
										(void 0 === r && (r = 0),
										null != r ? r > 0 : this.delay > 0)
									)
										return e.prototype.recycleAsyncId.call(this, t, n, r);
									var o,
										s = t.actions;
									null != n &&
										(null === (o = s[s.length - 1]) || void 0 === o
											? void 0
											: o.id) !== n &&
										(i.immediateProvider.clearImmediate(n),
										t._scheduled === n && (t._scheduled = void 0));
								}),
								t
							);
						})(o.AsyncAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsapScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AsapScheduler", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t() {
								return (null !== e && e.apply(this, arguments)) || this;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.flush = function (e) {
									this._active = !0;
									var t,
										n = this._scheduled;
									this._scheduled = void 0;
									var r = this.actions;
									e = e || r.shift();
									do if ((t = e.execute(e.state, e.delay))) break;
									while ((e = r[0]) && e.id === n && r.shift());
									if (((this._active = !1), t)) {
										for (; (e = r[0]) && e.id === n && r.shift(); )
											e.unsubscribe();
										throw t;
									}
								}),
								t
							);
						})(
							n(
								"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js",
							).AsyncScheduler,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AsyncAction", {
							enumerable: !0,
							get: function () {
								return l;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/Action.js",
						),
						i = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/intervalProvider.js",
						),
						s = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js",
						),
						l = (function (e) {
							function t(t, n) {
								var r = e.call(this, t, n) || this;
								return (r.scheduler = t), (r.work = n), (r.pending = !1), r;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.schedule = function (e, t) {
									if ((void 0 === t && (t = 0), this.closed)) return this;
									this.state = e;
									var n,
										r = this.id,
										o = this.scheduler;
									return (
										null != r && (this.id = this.recycleAsyncId(o, r, t)),
										(this.pending = !0),
										(this.delay = t),
										(this.id =
											null !== (n = this.id) && void 0 !== n
												? n
												: this.requestAsyncId(o, this.id, t)),
										this
									);
								}),
								(t.prototype.requestAsyncId = function (e, t, n) {
									return (
										void 0 === n && (n = 0),
										i.intervalProvider.setInterval(e.flush.bind(e, this), n)
									);
								}),
								(t.prototype.recycleAsyncId = function (e, t, n) {
									if (
										(void 0 === n && (n = 0),
										null != n && this.delay === n && !1 === this.pending)
									)
										return t;
									null != t && i.intervalProvider.clearInterval(t);
								}),
								(t.prototype.execute = function (e, t) {
									if (this.closed) return Error("executing a cancelled action");
									this.pending = !1;
									var n = this._execute(e, t);
									if (n) return n;
									!1 === this.pending &&
										null != this.id &&
										(this.id = this.recycleAsyncId(
											this.scheduler,
											this.id,
											null,
										));
								}),
								(t.prototype._execute = function (e, t) {
									var n,
										r = !1;
									try {
										this.work(e);
									} catch (e) {
										(r = !0),
											(n = e || Error("Scheduled action threw falsy error"));
									}
									if (r) return this.unsubscribe(), n;
								}),
								(t.prototype.unsubscribe = function () {
									if (!this.closed) {
										var t = this.id,
											n = this.scheduler,
											r = n.actions;
										(this.work = this.state = this.scheduler = null),
											(this.pending = !1),
											(0, s.arrRemove)(r, this),
											null != t && (this.id = this.recycleAsyncId(n, t, null)),
											(this.delay = null),
											e.prototype.unsubscribe.call(this);
									}
								}),
								t
							);
						})(o.Action);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "AsyncScheduler", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n("../../node_modules/rxjs/dist/esm5/internal/Scheduler.js"),
						i = (function (e) {
							function t(t, n) {
								void 0 === n && (n = o.Scheduler.now);
								var r = e.call(this, t, n) || this;
								return (r.actions = []), (r._active = !1), r;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.flush = function (e) {
									var t,
										n = this.actions;
									if (this._active) {
										n.push(e);
										return;
									}
									this._active = !0;
									do if ((t = e.execute(e.state, e.delay))) break;
									while ((e = n.shift()));
									if (((this._active = !1), t)) {
										for (; (e = n.shift()); ) e.unsubscribe();
										throw t;
									}
								}),
								t
							);
						})(o.Scheduler);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/QueueAction.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "QueueAction", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t(t, n) {
								var r = e.call(this, t, n) || this;
								return (r.scheduler = t), (r.work = n), r;
							}
							return (
								(0, r.__extends)(t, e),
								(t.prototype.schedule = function (t, n) {
									return (void 0 === n && (n = 0), n > 0)
										? e.prototype.schedule.call(this, t, n)
										: ((this.delay = n),
										  (this.state = t),
										  this.scheduler.flush(this),
										  this);
								}),
								(t.prototype.execute = function (t, n) {
									return n > 0 || this.closed
										? e.prototype.execute.call(this, t, n)
										: this._execute(t, n);
								}),
								(t.prototype.requestAsyncId = function (t, n, r) {
									return (void 0 === r && (r = 0),
									(null != r && r > 0) || (null == r && this.delay > 0))
										? e.prototype.requestAsyncId.call(this, t, n, r)
										: (t.flush(this), 0);
								}),
								t
							);
						})(
							n(
								"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js",
							).AsyncAction,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/QueueScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "QueueScheduler", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = (function (e) {
							function t() {
								return (null !== e && e.apply(this, arguments)) || this;
							}
							return (0, r.__extends)(t, e), t;
						})(
							n(
								"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js",
							).AsyncScheduler,
						);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/VirtualTimeScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js",
						),
						i = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js");
					!(function (e) {
						function t(t, n) {
							void 0 === t && (t = s), void 0 === n && (n = 1 / 0);
							var r =
								e.call(this, t, function () {
									return r.frame;
								}) || this;
							return (r.maxFrames = n), (r.frame = 0), (r.index = -1), r;
						}
						(0, r.__extends)(t, e),
							(t.prototype.flush = function () {
								for (
									var e, t, n = this.actions, r = this.maxFrames;
									(t = n[0]) &&
									t.delay <= r &&
									(n.shift(),
									(this.frame = t.delay),
									!(e = t.execute(t.state, t.delay)));
								);
								if (e) {
									for (; (t = n.shift()); ) t.unsubscribe();
									throw e;
								}
							}),
							(t.frameTimeFactor = 10);
					})(
						n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js",
						).AsyncScheduler,
					);
					var s = (function (e) {
						function t(t, n, r) {
							void 0 === r && (r = t.index += 1);
							var o = e.call(this, t, n) || this;
							return (
								(o.scheduler = t),
								(o.work = n),
								(o.index = r),
								(o.active = !0),
								(o.index = t.index = r),
								o
							);
						}
						return (
							(0, r.__extends)(t, e),
							(t.prototype.schedule = function (n, r) {
								if ((void 0 === r && (r = 0), !Number.isFinite(r)))
									return i.Subscription.EMPTY;
								if (!this.id) return e.prototype.schedule.call(this, n, r);
								this.active = !1;
								var o = new t(this.scheduler, this.work);
								return this.add(o), o.schedule(n, r);
							}),
							(t.prototype.requestAsyncId = function (e, n, r) {
								void 0 === r && (r = 0), (this.delay = e.frame + r);
								var o = e.actions;
								return o.push(this), o.sort(t.sortActions), 1;
							}),
							(t.prototype.recycleAsyncId = function (e, t, n) {
								void 0 === n && (n = 0);
							}),
							(t.prototype._execute = function (t, n) {
								if (!0 === this.active)
									return e.prototype._execute.call(this, t, n);
							}),
							(t.sortActions = function (e, t) {
								if (e.delay === t.delay)
									return e.index === t.index ? 0 : e.index > t.index ? 1 : -1;
								if (e.delay > t.delay) return 1;
								return -1;
							}),
							t
						);
					})(o.AsyncAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/animationFrame.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/AnimationFrameAction.js",
					);
					new (n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/AnimationFrameScheduler.js",
					).AnimationFrameScheduler)(r.AnimationFrameAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/animationFrameProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "animationFrameProvider", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n("../../node_modules/rxjs/dist/esm5/internal/Subscription.js"),
						i = {
							schedule: function (e) {
								var t = requestAnimationFrame,
									n = cancelAnimationFrame,
									r = i.delegate;
								r &&
									((t = r.requestAnimationFrame), (n = r.cancelAnimationFrame));
								var s = t(function (t) {
									(n = void 0), e(t);
								});
								return new o.Subscription(function () {
									return null == n ? void 0 : n(s);
								});
							},
							requestAnimationFrame: function () {
								for (var e = [], t = 0; t < arguments.length; t++)
									e[t] = arguments[t];
								var n = i.delegate;
								return (
									(null == n ? void 0 : n.requestAnimationFrame) ||
									requestAnimationFrame
								).apply(void 0, (0, r.__spreadArray)([], (0, r.__read)(e)));
							},
							cancelAnimationFrame: function () {
								for (var e = [], t = 0; t < arguments.length; t++)
									e[t] = arguments[t];
								var n = i.delegate;
								return (
									(null == n ? void 0 : n.cancelAnimationFrame) ||
									cancelAnimationFrame
								).apply(void 0, (0, r.__spreadArray)([], (0, r.__read)(e)));
							},
							delegate: void 0,
						};
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/asap.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				var r = n(
					"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsapAction.js",
				);
				new (n(
					"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsapScheduler.js",
				).AsapScheduler)(r.AsapAction);
			},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/async.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncAction.js",
					);
					new (n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/AsyncScheduler.js",
					).AsyncScheduler)(r.AsyncAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/dateTimestampProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "dateTimestampProvider", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = {
						now: function () {
							return (r.delegate || Date).now();
						},
						delegate: void 0,
					};
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/immediateProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "immediateProvider", {
							enumerable: !0,
							get: function () {
								return l;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/Immediate.js",
						),
						i = o.Immediate.setImmediate,
						s = o.Immediate.clearImmediate,
						l = {
							setImmediate: function () {
								for (var e = [], t = 0; t < arguments.length; t++)
									e[t] = arguments[t];
								var n = l.delegate;
								return ((null == n ? void 0 : n.setImmediate) || i).apply(
									void 0,
									(0, r.__spreadArray)([], (0, r.__read)(e)),
								);
							},
							clearImmediate: function (e) {
								var t = l.delegate;
								return ((null == t ? void 0 : t.clearImmediate) || s)(e);
							},
							delegate: void 0,
						};
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/intervalProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "intervalProvider", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = {
							setInterval: function (e, t) {
								for (var n = [], i = 2; i < arguments.length; i++)
									n[i - 2] = arguments[i];
								var s = o.delegate;
								return (null == s ? void 0 : s.setInterval)
									? s.setInterval.apply(
											s,
											(0, r.__spreadArray)([e, t], (0, r.__read)(n)),
									  )
									: setInterval.apply(
											void 0,
											(0, r.__spreadArray)([e, t], (0, r.__read)(n)),
									  );
							},
							clearInterval: function (e) {
								var t = o.delegate;
								return (
									(null == t ? void 0 : t.clearInterval) || clearInterval
								)(e);
							},
							delegate: void 0,
						};
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/performanceTimestampProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "performanceTimestampProvider", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = {
						now: function () {
							return (r.delegate || performance).now();
						},
						delegate: void 0,
					};
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/queue.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/QueueAction.js",
					);
					new (n(
						"../../node_modules/rxjs/dist/esm5/internal/scheduler/QueueScheduler.js",
					).QueueScheduler)(r.QueueAction);
				},
			"../../node_modules/rxjs/dist/esm5/internal/scheduler/timeoutProvider.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "timeoutProvider", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = {
							setTimeout: function (e, t) {
								for (var n = [], i = 2; i < arguments.length; i++)
									n[i - 2] = arguments[i];
								var s = o.delegate;
								return (null == s ? void 0 : s.setTimeout)
									? s.setTimeout.apply(
											s,
											(0, r.__spreadArray)([e, t], (0, r.__read)(n)),
									  )
									: setTimeout.apply(
											void 0,
											(0, r.__spreadArray)([e, t], (0, r.__read)(n)),
									  );
							},
							clearTimeout: function (e) {
								var t = o.delegate;
								return ((null == t ? void 0 : t.clearTimeout) || clearTimeout)(
									e,
								);
							},
							delegate: void 0,
						};
				},
			"../../node_modules/rxjs/dist/esm5/internal/symbol/iterator.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "iterator", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r =
						"function" == typeof Symbol && Symbol.iterator
							? Symbol.iterator
							: "@@iterator";
				},
			"../../node_modules/rxjs/dist/esm5/internal/symbol/observable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "observable", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r =
						("function" == typeof Symbol && Symbol.observable) ||
						"@@observable";
				},
			"../../node_modules/rxjs/dist/esm5/internal/types.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/ArgumentOutOfRangeError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						(0,
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
						).createErrorClass)(function (e) {
							return function () {
								e(this),
									(this.name = "ArgumentOutOfRangeError"),
									(this.message = "argument out of range");
							};
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/EmptyError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "EmptyError", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = (0,
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
					).createErrorClass)(function (e) {
						return function () {
							e(this),
								(this.name = "EmptyError"),
								(this.message = "no elements in sequence");
						};
					});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/Immediate.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "Immediate", {
						enumerable: !0,
						get: function () {
							return l;
						},
					});
				var r,
					o = 1,
					i = {};
				function s(e) {
					return e in i && (delete i[e], !0);
				}
				var l = {
					setImmediate: function (e) {
						var t = o++;
						return (
							(i[t] = !0),
							!r && (r = Promise.resolve()),
							r.then(function () {
								return s(t) && e();
							}),
							t
						);
					},
					clearImmediate: function (e) {
						s(e);
					},
				};
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/NotFoundError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						(0,
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
						).createErrorClass)(function (e) {
							return function (t) {
								e(this), (this.name = "NotFoundError"), (this.message = t);
							};
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/ObjectUnsubscribedError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "ObjectUnsubscribedError", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = (0,
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
					).createErrorClass)(function (e) {
						return function () {
							e(this),
								(this.name = "ObjectUnsubscribedError"),
								(this.message = "object unsubscribed");
						};
					});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/SequenceError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						(0,
						n(
							"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
						).createErrorClass)(function (e) {
							return function (t) {
								e(this), (this.name = "SequenceError"), (this.message = t);
							};
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/UnsubscriptionError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "UnsubscriptionError", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = (0,
					n(
						"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js",
					).createErrorClass)(function (e) {
						return function (t) {
							e(this),
								(this.message = t
									? t.length +
									  " errors occurred during unsubscription:\n" +
									  t
											.map(function (e, t) {
												return t + 1 + ") " + e.toString();
											})
											.join("\n  ")
									: ""),
								(this.name = "UnsubscriptionError"),
								(this.errors = t);
						};
					});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/args.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					popResultSelector: function () {
						return s;
					},
					popScheduler: function () {
						return l;
					},
					popNumber: function () {
						return a;
					},
				});
				var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					),
					o = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isScheduler.js",
					);
				function i(e) {
					return e[e.length - 1];
				}
				function s(e) {
					return (0, r.isFunction)(i(e)) ? e.pop() : void 0;
				}
				function l(e) {
					return (0, o.isScheduler)(i(e)) ? e.pop() : void 0;
				}
				function a(e, t) {
					return "number" == typeof i(e) ? e.pop() : t;
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/argsArgArrayOrObject.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "argsArgArrayOrObject", {
							enumerable: !0,
							get: function () {
								return l;
							},
						});
					var r = Array.isArray,
						o = Object.getPrototypeOf,
						i = Object.prototype,
						s = Object.keys;
					function l(e) {
						if (1 === e.length) {
							var t = e[0];
							if (r(t)) return { args: t, keys: null };
							if (
								(function (e) {
									return e && "object" == typeof e && o(e) === i;
								})(t)
							) {
								var n = s(t);
								return {
									args: n.map(function (e) {
										return t[e];
									}),
									keys: n,
								};
							}
						}
						return { args: e, keys: null };
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/argsOrArgArray.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "argsOrArgArray", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = Array.isArray;
					function o(e) {
						return 1 === e.length && r(e[0]) ? e[0] : e;
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/arrRemove.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				function r(e, t) {
					if (e) {
						var n = e.indexOf(t);
						0 <= n && e.splice(n, 1);
					}
				}
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "arrRemove", {
						enumerable: !0,
						get: function () {
							return r;
						},
					});
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/createErrorClass.js":
				function (e, t, n) {
					"use strict";
					function r(e) {
						var t = e(function (e) {
							Error.call(e), (e.stack = Error().stack);
						});
						return (
							(t.prototype = Object.create(Error.prototype)),
							(t.prototype.constructor = t),
							t
						);
					}
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "createErrorClass", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/createObject.js":
				function (e, t, n) {
					"use strict";
					function r(e, t) {
						return e.reduce(function (e, n, r) {
							return (e[n] = t[r]), e;
						}, {});
					}
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "createObject", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/errorContext.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					!(function (e, t) {
						for (var n in t)
							Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
					})(t, {
						errorContext: function () {
							return i;
						},
						captureError: function () {
							return s;
						},
					});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/config.js"),
						o = null;
					function i(e) {
						if (r.config.useDeprecatedSynchronousErrorHandling) {
							var t = !o;
							if ((t && (o = { errorThrown: !1, error: null }), e(), t)) {
								var n = o,
									i = n.errorThrown,
									s = n.error;
								if (((o = null), i)) throw s;
							}
						} else e();
					}
					function s(e) {
						r.config.useDeprecatedSynchronousErrorHandling &&
							o &&
							((o.errorThrown = !0), (o.error = e));
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/executeSchedule.js":
				function (e, t, n) {
					"use strict";
					function r(e, t, n, r, o) {
						void 0 === r && (r = 0), void 0 === o && (o = !1);
						var i = t.schedule(function () {
							n(), o ? e.add(this.schedule(null, r)) : this.unsubscribe();
						}, r);
						if ((e.add(i), !o)) return i;
					}
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "executeSchedule", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/identity.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				function r(e) {
					return e;
				}
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "identity", {
						enumerable: !0,
						get: function () {
							return r;
						},
					});
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/isArrayLike.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isArrayLike", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
					var r = function (e) {
						return e && "number" == typeof e.length && "function" != typeof e;
					};
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isAsyncIterable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isAsyncIterable", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					);
					function o(e) {
						return (
							Symbol.asyncIterator &&
							(0, r.isFunction)(null == e ? void 0 : e[Symbol.asyncIterator])
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isDate.js": function (
				e,
				t,
				n,
			) {},
			"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js":
				function (e, t, n) {
					"use strict";
					function r(e) {
						return "function" == typeof e;
					}
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isFunction", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isInteropObservable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isInteropObservable", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/symbol/observable.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e) {
						return (0, o.isFunction)(e[r.observable]);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isIterable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isIterable", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n(
							"../../node_modules/rxjs/dist/esm5/internal/symbol/iterator.js",
						),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e) {
						return (0, o.isFunction)(null == e ? void 0 : e[r.iterator]);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isObservable.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isObservable", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/Observable.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e) {
						return (
							!!e &&
							(e instanceof r.Observable ||
								((0, o.isFunction)(e.lift) && (0, o.isFunction)(e.subscribe)))
						);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isPromise.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "isPromise", {
						enumerable: !0,
						get: function () {
							return o;
						},
					});
				var r = n(
					"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
				);
				function o(e) {
					return (0, r.isFunction)(null == e ? void 0 : e.then);
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/isReadableStreamLike.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 });
					!(function (e, t) {
						for (var n in t)
							Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
					})(t, {
						readableStreamLikeToAsyncGenerator: function () {
							return i;
						},
						isReadableStreamLike: function () {
							return s;
						},
					});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
						);
					function i(e) {
						return (0, r.__asyncGenerator)(this, arguments, function () {
							var t, n, o;
							return (0, r.__generator)(this, function (i) {
								switch (i.label) {
									case 0:
										(t = e.getReader()), (i.label = 1);
									case 1:
										i.trys.push([1, , 9, 10]), (i.label = 2);
									case 2:
										return [4, (0, r.__await)(t.read())];
									case 3:
										if (((o = (n = i.sent()).value), !n.done)) return [3, 5];
										return [4, (0, r.__await)(void 0)];
									case 4:
										return [2, i.sent()];
									case 5:
										return [4, (0, r.__await)(o)];
									case 6:
										return [4, i.sent()];
									case 7:
										return i.sent(), [3, 2];
									case 8:
										return [3, 10];
									case 9:
										return t.releaseLock(), [7];
									case 10:
										return [2];
								}
							});
						});
					}
					function s(e) {
						return (0, o.isFunction)(null == e ? void 0 : e.getReader);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/isScheduler.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "isScheduler", {
							enumerable: !0,
							get: function () {
								return o;
							},
						});
					var r = n(
						"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
					);
					function o(e) {
						return e && (0, r.isFunction)(e.schedule);
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/lift.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					hasLift: function () {
						return o;
					},
					operate: function () {
						return i;
					},
				});
				var r = n(
					"../../node_modules/rxjs/dist/esm5/internal/util/isFunction.js",
				);
				function o(e) {
					return (0, r.isFunction)(null == e ? void 0 : e.lift);
				}
				function i(e) {
					return function (t) {
						if (o(t))
							return t.lift(function (t) {
								try {
									return e(t, this);
								} catch (e) {
									this.error(e);
								}
							});
						throw TypeError("Unable to lift unknown Observable type");
					};
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/mapOneOrManyArgs.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "mapOneOrManyArgs", {
							enumerable: !0,
							get: function () {
								return s;
							},
						});
					var r = n("../../node_modules/tslib/tslib.es6.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/operators/map.js",
						),
						i = Array.isArray;
					function s(e) {
						return (0, o.map)(function (t) {
							var n, o;
							return (
								(n = e),
								i((o = t))
									? n.apply(void 0, (0, r.__spreadArray)([], (0, r.__read)(o)))
									: n(o)
							);
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/noop.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				function r() {}
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "noop", {
						enumerable: !0,
						get: function () {
							return r;
						},
					});
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/not.js": function (
				e,
				t,
				n,
			) {},
			"../../node_modules/rxjs/dist/esm5/internal/util/pipe.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					pipe: function () {
						return o;
					},
					pipeFromArray: function () {
						return i;
					},
				});
				var r = n(
					"../../node_modules/rxjs/dist/esm5/internal/util/identity.js",
				);
				function o() {
					for (var e = [], t = 0; t < arguments.length; t++)
						e[t] = arguments[t];
					return i(e);
				}
				function i(e) {
					return 0 === e.length
						? r.identity
						: 1 === e.length
						? e[0]
						: function (t) {
								return e.reduce(function (e, t) {
									return t(e);
								}, t);
						  };
				}
			},
			"../../node_modules/rxjs/dist/esm5/internal/util/reportUnhandledError.js":
				function (e, t, n) {
					"use strict";
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "reportUnhandledError", {
							enumerable: !0,
							get: function () {
								return i;
							},
						});
					var r = n("../../node_modules/rxjs/dist/esm5/internal/config.js"),
						o = n(
							"../../node_modules/rxjs/dist/esm5/internal/scheduler/timeoutProvider.js",
						);
					function i(e) {
						o.timeoutProvider.setTimeout(function () {
							var t = r.config.onUnhandledError;
							if (t) t(e);
							else throw e;
						});
					}
				},
			"../../node_modules/rxjs/dist/esm5/internal/util/throwUnobservableError.js":
				function (e, t, n) {
					"use strict";
					function r(e) {
						return TypeError(
							"You provided " +
								(null !== e && "object" == typeof e
									? "an invalid object"
									: "'" + e + "'") +
								" where a stream was expected. You can provide an Observable, Promise, ReadableStream, Array, AsyncIterable, or Iterable.",
						);
					}
					Object.defineProperty(t, "__esModule", { value: !0 }),
						Object.defineProperty(t, "createInvalidObservableTypeError", {
							enumerable: !0,
							get: function () {
								return r;
							},
						});
				},
			"../../node_modules/rxjs/dist/esm5/operators/index.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					catchError: function () {
						return r.catchError;
					},
					concatMap: function () {
						return o.concatMap;
					},
					defaultIfEmpty: function () {
						return i.defaultIfEmpty;
					},
					filter: function () {
						return s.filter;
					},
					finalize: function () {
						return l.finalize;
					},
					first: function () {
						return a.first;
					},
					last: function () {
						return u.last;
					},
					map: function () {
						return d.map;
					},
					mapTo: function () {
						return c.mapTo;
					},
					mergeAll: function () {
						return f.mergeAll;
					},
					mergeMap: function () {
						return h.mergeMap;
					},
					refCount: function () {
						return p.refCount;
					},
					scan: function () {
						return m.scan;
					},
					share: function () {
						return g.share;
					},
					startWith: function () {
						return v.startWith;
					},
					switchMap: function () {
						return y.switchMap;
					},
					take: function () {
						return b.take;
					},
					takeLast: function () {
						return _.takeLast;
					},
					tap: function () {
						return j.tap;
					},
				}),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/audit.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/auditTime.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/buffer.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferCount.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferToggle.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/bufferWhen.js",
					);
				var r = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/catchError.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/combineAll.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatest.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/combineLatestWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/concat.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatAll.js",
					);
				var o = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/concatMap.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/concatMapTo.js",
				),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/concatWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/connect.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/count.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/debounce.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/debounceTime.js",
					);
				var i = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/defaultIfEmpty.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/delay.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/delayWhen.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/dematerialize.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/distinct.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilChanged.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/distinctUntilKeyChanged.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/elementAt.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/endWith.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/every.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/exhaust.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustAll.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/exhaustMap.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/expand.js");
				var s = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/filter.js",
					),
					l = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/finalize.js",
					);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/find.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/findIndex.js",
					);
				var a = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/first.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/groupBy.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/ignoreElements.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/isEmpty.js");
				var u = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/last.js",
					),
					d = n("../../node_modules/rxjs/dist/esm5/internal/operators/map.js"),
					c = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mapTo.js",
					);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/materialize.js",
				),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/max.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/merge.js");
				var f = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/mergeAll.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/flatMap.js");
				var h = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/mergeMap.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/mergeMapTo.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeScan.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/mergeWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/min.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/multicast.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/observeOn.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/onErrorResumeNextWith.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/pairwise.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/partition.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/pluck.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/publish.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishBehavior.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishLast.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/publishReplay.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/race.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/raceWith.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/reduce.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/repeat.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/repeatWhen.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/retry.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/retryWhen.js",
					);
				var p = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/refCount.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/sample.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/sampleTime.js",
					);
				var m = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/scan.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/sequenceEqual.js",
				);
				var g = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/share.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/shareReplay.js",
				),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/single.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/skip.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/skipLast.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/skipUntil.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/skipWhile.js",
					);
				var v = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/startWith.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/subscribeOn.js",
				),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchAll.js",
					);
				var y = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/switchMap.js",
				);
				n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/switchMapTo.js",
				),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/switchScan.js",
					);
				var b = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/take.js",
					),
					_ = n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/takeLast.js",
					);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/takeUntil.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/takeWhile.js",
					);
				var j = n(
					"../../node_modules/rxjs/dist/esm5/internal/operators/tap.js",
				);
				n("../../node_modules/rxjs/dist/esm5/internal/operators/throttle.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/throttleTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/throwIfEmpty.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timeInterval.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/timeout.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timeoutWith.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/timestamp.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/toArray.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/window.js"),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowCount.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowTime.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowToggle.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/windowWhen.js",
					),
					n(
						"../../node_modules/rxjs/dist/esm5/internal/operators/withLatestFrom.js",
					),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/zip.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/zipAll.js"),
					n("../../node_modules/rxjs/dist/esm5/internal/operators/zipWith.js");
			},
			"../../node_modules/tslib/tslib.es6.js": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					__extends: function () {
						return o;
					},
					__awaiter: function () {
						return s;
					},
					__generator: function () {
						return l;
					},
					__values: function () {
						return a;
					},
					__read: function () {
						return u;
					},
					__spreadArray: function () {
						return d;
					},
					__await: function () {
						return c;
					},
					__asyncGenerator: function () {
						return f;
					},
					__asyncValues: function () {
						return h;
					},
				});
				var r = function (e, t) {
					return (r =
						Object.setPrototypeOf ||
						({ __proto__: [] } instanceof Array &&
							function (e, t) {
								e.__proto__ = t;
							}) ||
						function (e, t) {
							for (var n in t)
								Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n]);
						})(e, t);
				};
				function o(e, t) {
					if ("function" != typeof t && null !== t)
						throw TypeError(
							"Class extends value " +
								String(t) +
								" is not a constructor or null",
						);
					function n() {
						this.constructor = e;
					}
					r(e, t),
						(e.prototype =
							null === t
								? Object.create(t)
								: ((n.prototype = t.prototype), new n()));
				}
				var i = function () {
					return (i =
						Object.assign ||
						function (e) {
							for (var t, n = 1, r = arguments.length; n < r; n++)
								for (var o in ((t = arguments[n]), t))
									Object.prototype.hasOwnProperty.call(t, o) && (e[o] = t[o]);
							return e;
						}).apply(this, arguments);
				};
				function s(e, t, n, r) {
					return new (n || (n = Promise))(function (o, i) {
						function s(e) {
							try {
								a(r.next(e));
							} catch (e) {
								i(e);
							}
						}
						function l(e) {
							try {
								a(r.throw(e));
							} catch (e) {
								i(e);
							}
						}
						function a(e) {
							var t;
							e.done
								? o(e.value)
								: ((t = e.value) instanceof n
										? t
										: new n(function (e) {
												e(t);
										  })
								  ).then(s, l);
						}
						a((r = r.apply(e, t || [])).next());
					});
				}
				function l(e, t) {
					var n,
						r,
						o,
						i,
						s = {
							label: 0,
							sent: function () {
								if (1 & o[0]) throw o[1];
								return o[1];
							},
							trys: [],
							ops: [],
						};
					return (
						(i = { next: l(0), throw: l(1), return: l(2) }),
						"function" == typeof Symbol &&
							(i[Symbol.iterator] = function () {
								return this;
							}),
						i
					);
					function l(l) {
						return function (a) {
							return (function (l) {
								if (n) throw TypeError("Generator is already executing.");
								for (; i && ((i = 0), l[0] && (s = 0)), s; )
									try {
										if (
											((n = 1),
											r &&
												(o =
													2 & l[0]
														? r.return
														: l[0]
														? r.throw || ((o = r.return) && o.call(r), 0)
														: r.next) &&
												!(o = o.call(r, l[1])).done)
										)
											return o;
										switch (((r = 0), o && (l = [2 & l[0], o.value]), l[0])) {
											case 0:
											case 1:
												o = l;
												break;
											case 4:
												return s.label++, { value: l[1], done: !1 };
											case 5:
												s.label++, (r = l[1]), (l = [0]);
												continue;
											case 7:
												(l = s.ops.pop()), s.trys.pop();
												continue;
											default:
												if (
													!(o = (o = s.trys).length > 0 && o[o.length - 1]) &&
													(6 === l[0] || 2 === l[0])
												) {
													s = 0;
													continue;
												}
												if (
													3 === l[0] &&
													(!o || (l[1] > o[0] && l[1] < o[3]))
												) {
													s.label = l[1];
													break;
												}
												if (6 === l[0] && s.label < o[1]) {
													(s.label = o[1]), (o = l);
													break;
												}
												if (o && s.label < o[2]) {
													(s.label = o[2]), s.ops.push(l);
													break;
												}
												o[2] && s.ops.pop(), s.trys.pop();
												continue;
										}
										l = t.call(e, s);
									} catch (e) {
										(l = [6, e]), (r = 0);
									} finally {
										n = o = 0;
									}
								if (5 & l[0]) throw l[1];
								return { value: l[0] ? l[1] : void 0, done: !0 };
							})([l, a]);
						};
					}
				}
				function a(e) {
					var t = "function" == typeof Symbol && Symbol.iterator,
						n = t && e[t],
						r = 0;
					if (n) return n.call(e);
					if (e && "number" == typeof e.length)
						return {
							next: function () {
								return (
									e && r >= e.length && (e = void 0),
									{ value: e && e[r++], done: !e }
								);
							},
						};
					throw TypeError(
						t ? "Object is not iterable." : "Symbol.iterator is not defined.",
					);
				}
				function u(e, t) {
					var n = "function" == typeof Symbol && e[Symbol.iterator];
					if (!n) return e;
					var r,
						o,
						i = n.call(e),
						s = [];
					try {
						for (; (void 0 === t || t-- > 0) && !(r = i.next()).done; )
							s.push(r.value);
					} catch (e) {
						o = { error: e };
					} finally {
						try {
							r && !r.done && (n = i.return) && n.call(i);
						} finally {
							if (o) throw o.error;
						}
					}
					return s;
				}
				function d(e, t, n) {
					if (n || 2 == arguments.length)
						for (var r, o = 0, i = t.length; o < i; o++)
							(r || !(o in t)) &&
								(!r && (r = Array.prototype.slice.call(t, 0, o)),
								(r[o] = t[o]));
					return e.concat(r || Array.prototype.slice.call(t));
				}
				function c(e) {
					return this instanceof c ? ((this.v = e), this) : new c(e);
				}
				function f(e, t, n) {
					if (!Symbol.asyncIterator)
						throw TypeError("Symbol.asyncIterator is not defined.");
					var r,
						o = n.apply(e, t || []),
						i = [];
					return (
						(r = {}),
						s("next"),
						s("throw"),
						s("return"),
						(r[Symbol.asyncIterator] = function () {
							return this;
						}),
						r
					);
					function s(e) {
						o[e] &&
							(r[e] = function (t) {
								return new Promise(function (n, r) {
									i.push([e, t, n, r]) > 1 || l(e, t);
								});
							});
					}
					function l(e, t) {
						try {
							(function (e) {
								e.value instanceof c
									? Promise.resolve(e.value.v).then(a, u)
									: d(i[0][2], e);
							})(o[e](t));
						} catch (e) {
							d(i[0][3], e);
						}
					}
					function a(e) {
						l("next", e);
					}
					function u(e) {
						l("throw", e);
					}
					function d(e, t) {
						e(t), i.shift(), i.length && l(i[0][0], i[0][1]);
					}
				}
				function h(e) {
					if (!Symbol.asyncIterator)
						throw TypeError("Symbol.asyncIterator is not defined.");
					var t,
						n = e[Symbol.asyncIterator];
					return n
						? n.call(e)
						: ((e = a(e)),
						  (t = {}),
						  r("next"),
						  r("throw"),
						  r("return"),
						  (t[Symbol.asyncIterator] = function () {
								return this;
						  }),
						  t);
					function r(n) {
						t[n] =
							e[n] &&
							function (t) {
								return new Promise(function (r, o) {
									(function (e, t, n, r) {
										Promise.resolve(r).then(function (t) {
											e({ value: t, done: n });
										}, t);
									})(r, o, (t = e[n](t)).done, t.value);
								});
							};
					}
				}
			},
			"../../node_modules/@angular/common/fesm2022/common.mjs": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					CommonModule: function () {
						return eR;
					},
					DOCUMENT: function () {
						return d;
					},
					HashLocationStrategy: function () {
						return j;
					},
					LOCATION_INITIALIZED: function () {
						return f;
					},
					Location: function () {
						return x;
					},
					LocationStrategy: function () {
						return y;
					},
					NgSwitch: function () {
						return ej;
					},
					NgSwitchCase: function () {
						return ex;
					},
					NgSwitchDefault: function () {
						return eD;
					},
					PathLocationStrategy: function () {
						return _;
					},
					ViewportScroller: function () {
						return e$;
					},
					XhrFactory: function () {
						return eU;
					},
					isPlatformServer: function () {
						return eL;
					},
					ɵDomAdapter: function () {
						return u;
					},
					ɵPLATFORM_BROWSER_ID: function () {
						return eN;
					},
					ɵgetDOM: function () {
						return l;
					},
					ɵparseCookieValue: function () {
						return ef;
					},
					ɵsetRootDomAdapter: function () {
						return a;
					},
				});
				var r = n("../../node_modules/@swc/helpers/esm/_object_spread.js"),
					o = n("../../node_modules/@swc/helpers/esm/_object_spread_props.js"),
					i = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs"));
				let s = null;
				function l() {
					return s;
				}
				function a(e) {
					!s && (s = e);
				}
				class u {}
				let d = new i.InjectionToken("DocumentToken"),
					c = (() => {
						class e {
							historyGo(e) {
								throw Error("Not implemented");
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: function () {
									return (function () {
										return (0, i.ɵɵinject)(h);
									})();
								},
								providedIn: "platform",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let f = new i.InjectionToken("Location Initialized"),
					h = (() => {
						class e extends c {
							getBaseHrefFromDOM() {
								return s.getBaseHref(this._doc);
							}
							onPopState(e) {
								let t = s.getGlobalEventTarget(this._doc, "window");
								return (
									t.addEventListener("popstate", e, !1),
									() => t.removeEventListener("popstate", e)
								);
							}
							onHashChange(e) {
								let t = s.getGlobalEventTarget(this._doc, "window");
								return (
									t.addEventListener("hashchange", e, !1),
									() => t.removeEventListener("hashchange", e)
								);
							}
							get href() {
								return this._location.href;
							}
							get protocol() {
								return this._location.protocol;
							}
							get hostname() {
								return this._location.hostname;
							}
							get port() {
								return this._location.port;
							}
							get pathname() {
								return this._location.pathname;
							}
							get search() {
								return this._location.search;
							}
							get hash() {
								return this._location.hash;
							}
							set pathname(e) {
								this._location.pathname = e;
							}
							pushState(e, t, n) {
								p()
									? this._history.pushState(e, t, n)
									: (this._location.hash = n);
							}
							replaceState(e, t, n) {
								p()
									? this._history.replaceState(e, t, n)
									: (this._location.hash = n);
							}
							forward() {
								this._history.forward();
							}
							back() {
								this._history.back();
							}
							historyGo(e = 0) {
								this._history.go(e);
							}
							getState() {
								return this._history.state;
							}
							constructor(e) {
								super(),
									(this._doc = e),
									(this._location = window.location),
									(this._history = window.history);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(i.ɵɵinject(d));
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: function () {
									return (function () {
										return new h((0, i.ɵɵinject)(d));
									})();
								},
								providedIn: "platform",
							})),
							e
						);
					})();
				function p() {
					return !!window.history.pushState;
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				function m(e, t) {
					if (0 == e.length) return t;
					if (0 == t.length) return e;
					let n = 0;
					return (e.endsWith("/") && n++, t.startsWith("/") && n++, 2 == n)
						? e + t.substring(1)
						: 1 == n
						? e + t
						: e + "/" + t;
				}
				function g(e) {
					let t = e.match(/#|\?|$/),
						n = (t && t.index) || e.length,
						r = n - ("/" === e[n - 1] ? 1 : 0);
					return e.slice(0, r) + e.slice(n);
				}
				function v(e) {
					return e && "?" !== e[0] ? "?" + e : e;
				}
				let y = (() => {
					class e {
						historyGo(e) {
							throw Error("Not implemented");
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function () {
								return (0, i.inject)(_);
							},
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let b = new i.InjectionToken("appBaseHref"),
					_ = (() => {
						class e extends y {
							ngOnDestroy() {
								for (; this._removeListenerFns.length; )
									this._removeListenerFns.pop()();
							}
							onPopState(e) {
								this._removeListenerFns.push(
									this._platformLocation.onPopState(e),
									this._platformLocation.onHashChange(e),
								);
							}
							getBaseHref() {
								return this._baseHref;
							}
							prepareExternalUrl(e) {
								return m(this._baseHref, e);
							}
							path(e = !1) {
								let t =
										this._platformLocation.pathname +
										v(this._platformLocation.search),
									n = this._platformLocation.hash;
								return n && e ? `${t}${n}` : t;
							}
							pushState(e, t, n, r) {
								let o = this.prepareExternalUrl(n + v(r));
								this._platformLocation.pushState(e, t, o);
							}
							replaceState(e, t, n, r) {
								let o = this.prepareExternalUrl(n + v(r));
								this._platformLocation.replaceState(e, t, o);
							}
							forward() {
								this._platformLocation.forward();
							}
							back() {
								this._platformLocation.back();
							}
							getState() {
								return this._platformLocation.getState();
							}
							historyGo(e = 0) {
								var t, n;
								null === (n = (t = this._platformLocation).historyGo) ||
									void 0 === n ||
									n.call(t, e);
							}
							constructor(e, t) {
								var n, r, o;
								super(),
									(this._platformLocation = e),
									(this._removeListenerFns = []),
									(this._baseHref =
										null !==
											(o =
												null !==
													(r =
														null != t
															? t
															: this._platformLocation.getBaseHrefFromDOM()) &&
												void 0 !== r
													? r
													: null === (n = (0, i.inject)(d).location) ||
													  void 0 === n
													? void 0
													: n.origin) && void 0 !== o
											? o
											: "");
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(i.ɵɵinject(c), i.ɵɵinject(b, 8));
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let j = (() => {
					class e extends y {
						ngOnDestroy() {
							for (; this._removeListenerFns.length; )
								this._removeListenerFns.pop()();
						}
						onPopState(e) {
							this._removeListenerFns.push(
								this._platformLocation.onPopState(e),
								this._platformLocation.onHashChange(e),
							);
						}
						getBaseHref() {
							return this._baseHref;
						}
						path(e = !1) {
							let t = this._platformLocation.hash;
							return null == t && (t = "#"), t.length > 0 ? t.substring(1) : t;
						}
						prepareExternalUrl(e) {
							let t = m(this._baseHref, e);
							return t.length > 0 ? "#" + t : t;
						}
						pushState(e, t, n, r) {
							let o = this.prepareExternalUrl(n + v(r));
							0 == o.length && (o = this._platformLocation.pathname),
								this._platformLocation.pushState(e, t, o);
						}
						replaceState(e, t, n, r) {
							let o = this.prepareExternalUrl(n + v(r));
							0 == o.length && (o = this._platformLocation.pathname),
								this._platformLocation.replaceState(e, t, o);
						}
						forward() {
							this._platformLocation.forward();
						}
						back() {
							this._platformLocation.back();
						}
						getState() {
							return this._platformLocation.getState();
						}
						historyGo(e = 0) {
							var t, n;
							null === (n = (t = this._platformLocation).historyGo) ||
								void 0 === n ||
								n.call(t, e);
						}
						constructor(e, t) {
							super(),
								(this._platformLocation = e),
								(this._baseHref = ""),
								(this._removeListenerFns = []),
								null != t && (this._baseHref = t);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵinject(c), i.ɵɵinject(b, 8));
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let x = (() => {
					class e {
						ngOnDestroy() {
							var e;
							null === (e = this._urlChangeSubscription) ||
								void 0 === e ||
								e.unsubscribe(),
								(this._urlChangeListeners = []);
						}
						path(e = !1) {
							return this.normalize(this._locationStrategy.path(e));
						}
						getState() {
							return this._locationStrategy.getState();
						}
						isCurrentPathEqualTo(e, t = "") {
							return this.path() == this.normalize(e + v(t));
						}
						normalize(t) {
							return e.stripTrailingSlash(
								(function (e, t) {
									if (!e || !t.startsWith(e)) return t;
									let n = t.substring(e.length);
									return "" === n || ["/", ";", "?", "#"].includes(n[0])
										? n
										: t;
								})(this._basePath, D(t)),
							);
						}
						prepareExternalUrl(e) {
							return (
								e && "/" !== e[0] && (e = "/" + e),
								this._locationStrategy.prepareExternalUrl(e)
							);
						}
						go(e, t = "", n = null) {
							this._locationStrategy.pushState(n, "", e, t),
								this._notifyUrlChangeListeners(
									this.prepareExternalUrl(e + v(t)),
									n,
								);
						}
						replaceState(e, t = "", n = null) {
							this._locationStrategy.replaceState(n, "", e, t),
								this._notifyUrlChangeListeners(
									this.prepareExternalUrl(e + v(t)),
									n,
								);
						}
						forward() {
							this._locationStrategy.forward();
						}
						back() {
							this._locationStrategy.back();
						}
						historyGo(e = 0) {
							var t, n;
							null === (n = (t = this._locationStrategy).historyGo) ||
								void 0 === n ||
								n.call(t, e);
						}
						onUrlChange(e) {
							return (
								this._urlChangeListeners.push(e),
								!this._urlChangeSubscription &&
									(this._urlChangeSubscription = this.subscribe((e) => {
										this._notifyUrlChangeListeners(e.url, e.state);
									})),
								() => {
									let t = this._urlChangeListeners.indexOf(e);
									if (
										(this._urlChangeListeners.splice(t, 1),
										0 === this._urlChangeListeners.length)
									) {
										var n;
										null === (n = this._urlChangeSubscription) ||
											void 0 === n ||
											n.unsubscribe(),
											(this._urlChangeSubscription = null);
									}
								}
							);
						}
						_notifyUrlChangeListeners(e = "", t) {
							this._urlChangeListeners.forEach((n) => n(e, t));
						}
						subscribe(e, t, n) {
							return this._subject.subscribe({
								next: e,
								error: t,
								complete: n,
							});
						}
						constructor(e) {
							(this._subject = new i.EventEmitter()),
								(this._urlChangeListeners = []),
								(this._urlChangeSubscription = null),
								(this._locationStrategy = e);
							let t = this._locationStrategy.getBaseHref();
							(this._basePath = (function (e) {
								let t = RegExp("^(https?:)?//").test(e);
								if (t) {
									let [, t] = e.split(/\/\/[^\/]+/);
									return t;
								}
								return e;
							})(g(D(t)))),
								this._locationStrategy.onPopState((e) => {
									this._subject.emit({
										url: this.path(!0),
										pop: !0,
										state: e.state,
										type: e.type,
									});
								});
						}
					}
					return (
						(e.normalizeQueryParams = v),
						(e.joinWithSlash = m),
						(e.stripTrailingSlash = g),
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵinject(y));
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function () {
								return (function () {
									return new x((0, i.ɵɵinject)(y));
								})();
							},
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function D(e) {
					return e.replace(/\/index.html$/, "");
				}
				let w = {
					ADP: [void 0, void 0, 0],
					AFN: [void 0, "؋", 0],
					ALL: [void 0, void 0, 0],
					AMD: [void 0, "֏", 2],
					AOA: [void 0, "Kz"],
					ARS: [void 0, "$"],
					AUD: ["A$", "$"],
					AZN: [void 0, "₼"],
					BAM: [void 0, "KM"],
					BBD: [void 0, "$"],
					BDT: [void 0, "৳"],
					BHD: [void 0, void 0, 3],
					BIF: [void 0, void 0, 0],
					BMD: [void 0, "$"],
					BND: [void 0, "$"],
					BOB: [void 0, "Bs"],
					BRL: ["R$"],
					BSD: [void 0, "$"],
					BWP: [void 0, "P"],
					BYN: [void 0, void 0, 2],
					BYR: [void 0, void 0, 0],
					BZD: [void 0, "$"],
					CAD: ["CA$", "$", 2],
					CHF: [void 0, void 0, 2],
					CLF: [void 0, void 0, 4],
					CLP: [void 0, "$", 0],
					CNY: ["CN\xa5", "\xa5"],
					COP: [void 0, "$", 2],
					CRC: [void 0, "₡", 2],
					CUC: [void 0, "$"],
					CUP: [void 0, "$"],
					CZK: [void 0, "Kč", 2],
					DJF: [void 0, void 0, 0],
					DKK: [void 0, "kr", 2],
					DOP: [void 0, "$"],
					EGP: [void 0, "E\xa3"],
					ESP: [void 0, "₧", 0],
					EUR: ["€"],
					FJD: [void 0, "$"],
					FKP: [void 0, "\xa3"],
					GBP: ["\xa3"],
					GEL: [void 0, "₾"],
					GHS: [void 0, "GH₵"],
					GIP: [void 0, "\xa3"],
					GNF: [void 0, "FG", 0],
					GTQ: [void 0, "Q"],
					GYD: [void 0, "$", 2],
					HKD: ["HK$", "$"],
					HNL: [void 0, "L"],
					HRK: [void 0, "kn"],
					HUF: [void 0, "Ft", 2],
					IDR: [void 0, "Rp", 2],
					ILS: ["₪"],
					INR: ["₹"],
					IQD: [void 0, void 0, 0],
					IRR: [void 0, void 0, 0],
					ISK: [void 0, "kr", 0],
					ITL: [void 0, void 0, 0],
					JMD: [void 0, "$"],
					JOD: [void 0, void 0, 3],
					JPY: ["\xa5", void 0, 0],
					KHR: [void 0, "៛"],
					KMF: [void 0, "CF", 0],
					KPW: [void 0, "₩", 0],
					KRW: ["₩", void 0, 0],
					KWD: [void 0, void 0, 3],
					KYD: [void 0, "$"],
					KZT: [void 0, "₸"],
					LAK: [void 0, "₭", 0],
					LBP: [void 0, "L\xa3", 0],
					LKR: [void 0, "Rs"],
					LRD: [void 0, "$"],
					LTL: [void 0, "Lt"],
					LUF: [void 0, void 0, 0],
					LVL: [void 0, "Ls"],
					LYD: [void 0, void 0, 3],
					MGA: [void 0, "Ar", 0],
					MGF: [void 0, void 0, 0],
					MMK: [void 0, "K", 0],
					MNT: [void 0, "₮", 2],
					MRO: [void 0, void 0, 0],
					MUR: [void 0, "Rs", 2],
					MXN: ["MX$", "$"],
					MYR: [void 0, "RM"],
					NAD: [void 0, "$"],
					NGN: [void 0, "₦"],
					NIO: [void 0, "C$"],
					NOK: [void 0, "kr", 2],
					NPR: [void 0, "Rs"],
					NZD: ["NZ$", "$"],
					OMR: [void 0, void 0, 3],
					PHP: ["₱"],
					PKR: [void 0, "Rs", 2],
					PLN: [void 0, "zł"],
					PYG: [void 0, "₲", 0],
					RON: [void 0, "lei"],
					RSD: [void 0, void 0, 0],
					RUB: [void 0, "₽"],
					RWF: [void 0, "RF", 0],
					SBD: [void 0, "$"],
					SEK: [void 0, "kr", 2],
					SGD: [void 0, "$"],
					SHP: [void 0, "\xa3"],
					SLE: [void 0, void 0, 2],
					SLL: [void 0, void 0, 0],
					SOS: [void 0, void 0, 0],
					SRD: [void 0, "$"],
					SSP: [void 0, "\xa3"],
					STD: [void 0, void 0, 0],
					STN: [void 0, "Db"],
					SYP: [void 0, "\xa3", 0],
					THB: [void 0, "฿"],
					TMM: [void 0, void 0, 0],
					TND: [void 0, void 0, 3],
					TOP: [void 0, "T$"],
					TRL: [void 0, void 0, 0],
					TRY: [void 0, "₺"],
					TTD: [void 0, "$"],
					TWD: ["NT$", "$", 2],
					TZS: [void 0, void 0, 2],
					UAH: [void 0, "₴"],
					UGX: [void 0, void 0, 0],
					USD: ["$"],
					UYI: [void 0, void 0, 0],
					UYU: [void 0, "$"],
					UYW: [void 0, void 0, 4],
					UZS: [void 0, void 0, 2],
					VEF: [void 0, "Bs", 2],
					VND: ["₫", void 0, 0],
					VUV: [void 0, void 0, 0],
					XAF: ["FCFA", void 0, 0],
					XCD: ["EC$", "$"],
					XOF: ["F CFA", void 0, 0],
					XPF: ["CFPF", void 0, 0],
					XXX: ["\xa4"],
					YER: [void 0, void 0, 0],
					ZAR: [void 0, "R"],
					ZMK: [void 0, void 0, 0],
					ZMW: [void 0, "ZK"],
					ZWD: [void 0, void 0, 0],
				};
				var M =
						(((M = M || {})[(M.Decimal = 0)] = "Decimal"),
						(M[(M.Percent = 1)] = "Percent"),
						(M[(M.Currency = 2)] = "Currency"),
						(M[(M.Scientific = 3)] = "Scientific"),
						M),
					C =
						(((C = C || {})[(C.Zero = 0)] = "Zero"),
						(C[(C.One = 1)] = "One"),
						(C[(C.Two = 2)] = "Two"),
						(C[(C.Few = 3)] = "Few"),
						(C[(C.Many = 4)] = "Many"),
						(C[(C.Other = 5)] = "Other"),
						C),
					S =
						(((S = S || {})[(S.Format = 0)] = "Format"),
						(S[(S.Standalone = 1)] = "Standalone"),
						S),
					E =
						(((E = E || {})[(E.Narrow = 0)] = "Narrow"),
						(E[(E.Abbreviated = 1)] = "Abbreviated"),
						(E[(E.Wide = 2)] = "Wide"),
						(E[(E.Short = 3)] = "Short"),
						E),
					O =
						(((O = O || {})[(O.Short = 0)] = "Short"),
						(O[(O.Medium = 1)] = "Medium"),
						(O[(O.Long = 2)] = "Long"),
						(O[(O.Full = 3)] = "Full"),
						O),
					A =
						(((A = A || {})[(A.Decimal = 0)] = "Decimal"),
						(A[(A.Group = 1)] = "Group"),
						(A[(A.List = 2)] = "List"),
						(A[(A.PercentSign = 3)] = "PercentSign"),
						(A[(A.PlusSign = 4)] = "PlusSign"),
						(A[(A.MinusSign = 5)] = "MinusSign"),
						(A[(A.Exponential = 6)] = "Exponential"),
						(A[(A.SuperscriptingExponent = 7)] = "SuperscriptingExponent"),
						(A[(A.PerMille = 8)] = "PerMille"),
						(A[(A.Infinity = 9)] = "Infinity"),
						(A[(A.NaN = 10)] = "NaN"),
						(A[(A.TimeSeparator = 11)] = "TimeSeparator"),
						(A[(A.CurrencyDecimal = 12)] = "CurrencyDecimal"),
						(A[(A.CurrencyGroup = 13)] = "CurrencyGroup"),
						A),
					I =
						(((I = I || {})[(I.Sunday = 0)] = "Sunday"),
						(I[(I.Monday = 1)] = "Monday"),
						(I[(I.Tuesday = 2)] = "Tuesday"),
						(I[(I.Wednesday = 3)] = "Wednesday"),
						(I[(I.Thursday = 4)] = "Thursday"),
						(I[(I.Friday = 5)] = "Friday"),
						(I[(I.Saturday = 6)] = "Saturday"),
						I);
				function P(e, t) {
					let n = (0, i.ɵfindLocaleData)(e);
					return $(n[i.ɵLocaleDataIndex.DateFormat], t);
				}
				function T(e, t) {
					let n = (0, i.ɵfindLocaleData)(e);
					return $(n[i.ɵLocaleDataIndex.TimeFormat], t);
				}
				function k(e, t) {
					let n = (0, i.ɵfindLocaleData)(e),
						r = n[i.ɵLocaleDataIndex.DateTimeFormat];
					return $(r, t);
				}
				function F(e, t) {
					let n = (0, i.ɵfindLocaleData)(e),
						r = n[i.ɵLocaleDataIndex.NumberSymbols][t];
					if (void 0 === r) {
						if (t === A.CurrencyDecimal)
							return n[i.ɵLocaleDataIndex.NumberSymbols][A.Decimal];
						if (t === A.CurrencyGroup)
							return n[i.ɵLocaleDataIndex.NumberSymbols][A.Group];
					}
					return r;
				}
				function R(e, t) {
					let n = (0, i.ɵfindLocaleData)(e);
					return n[i.ɵLocaleDataIndex.NumberFormats][t];
				}
				let N = i.ɵgetLocalePluralCase;
				function L(e) {
					if (!e[i.ɵLocaleDataIndex.ExtraData])
						throw Error(
							`Missing extra locale data for the locale "${
								e[i.ɵLocaleDataIndex.LocaleId]
							}". Use "registerLocaleData" to load new data. See the "I18n guide" on angular.io to know more.`,
						);
				}
				function $(e, t) {
					for (let n = t; n > -1; n--) if (void 0 !== e[n]) return e[n];
					throw Error("Locale data API: locale data undefined");
				}
				function V(e) {
					let [t, n] = e.split(":");
					return { hours: +t, minutes: +n };
				}
				let B =
						/^(\d{4,})-?(\d\d)-?(\d\d)(?:T(\d\d)(?::?(\d\d)(?::?(\d\d)(?:\.(\d+))?)?)?(Z|([+-])(\d\d):?(\d\d))?)?$/,
					U = {},
					H =
						/((?:[^BEGHLMOSWYZabcdhmswyz']+)|(?:'(?:[^']|'')*')|(?:G{1,5}|y{1,4}|Y{1,4}|M{1,5}|L{1,5}|w{1,2}|W{1}|d{1,2}|E{1,6}|c{1,6}|a{1,5}|b{1,5}|B{1,5}|h{1,2}|H{1,2}|m{1,2}|s{1,2}|S{1,3}|z{1,4}|Z{1,5}|O{1,4}))([\s\S]*)/;
				var z =
						(((z = z || {})[(z.Short = 0)] = "Short"),
						(z[(z.ShortGMT = 1)] = "ShortGMT"),
						(z[(z.Long = 2)] = "Long"),
						(z[(z.Extended = 3)] = "Extended"),
						z),
					W =
						(((W = W || {})[(W.FullYear = 0)] = "FullYear"),
						(W[(W.Month = 1)] = "Month"),
						(W[(W.Date = 2)] = "Date"),
						(W[(W.Hours = 3)] = "Hours"),
						(W[(W.Minutes = 4)] = "Minutes"),
						(W[(W.Seconds = 5)] = "Seconds"),
						(W[(W.FractionalSeconds = 6)] = "FractionalSeconds"),
						(W[(W.Day = 7)] = "Day"),
						W),
					q =
						(((q = q || {})[(q.DayPeriods = 0)] = "DayPeriods"),
						(q[(q.Days = 1)] = "Days"),
						(q[(q.Months = 2)] = "Months"),
						(q[(q.Eras = 3)] = "Eras"),
						q);
				function G(e, t, n) {
					let r = new Date(0);
					return r.setFullYear(e, t, n), r.setHours(0, 0, 0), r;
				}
				function Z(e, t) {
					return (
						t &&
							(e = e.replace(/\{([^}]+)}/g, function (e, n) {
								return null != t && n in t ? t[n] : e;
							})),
						e
					);
				}
				function Y(e, t, n = "-", r, o) {
					let i = "";
					(e < 0 || (o && e <= 0)) && (o ? (e = -e + 1) : ((e = -e), (i = n)));
					let s = String(e);
					for (; s.length < t; ) s = "0" + s;
					return r && (s = s.slice(s.length - t)), i + s;
				}
				function Q(e, t, n = 0, r = !1, o = !1) {
					return function (i, s) {
						let l = (function (e, t) {
							switch (e) {
								case W.FullYear:
									return t.getFullYear();
								case W.Month:
									return t.getMonth();
								case W.Date:
									return t.getDate();
								case W.Hours:
									return t.getHours();
								case W.Minutes:
									return t.getMinutes();
								case W.Seconds:
									return t.getSeconds();
								case W.FractionalSeconds:
									return t.getMilliseconds();
								case W.Day:
									return t.getDay();
								default:
									throw Error(`Unknown DateType value "${e}".`);
							}
						})(e, i);
						if (((n > 0 || l > -n) && (l += n), e === W.Hours))
							0 === l && -12 === n && (l = 12);
						else if (e === W.FractionalSeconds)
							return (function (e, t) {
								let n = Y(e, 3);
								return n.substring(0, t);
							})(l, t);
						let a = F(s, A.MinusSign);
						return Y(l, t, a, r, o);
					};
				}
				function K(e, t, n = S.Format, r = !1) {
					return function (o, s) {
						return (function (e, t, n, r, o, s) {
							switch (n) {
								case q.Months:
									return (function (e, t, n) {
										let r = (0, i.ɵfindLocaleData)(e),
											o = [
												r[i.ɵLocaleDataIndex.MonthsFormat],
												r[i.ɵLocaleDataIndex.MonthsStandalone],
											],
											s = $(o, t);
										return $(s, n);
									})(t, o, r)[e.getMonth()];
								case q.Days:
									return (function (e, t, n) {
										let r = (0, i.ɵfindLocaleData)(e),
											o = [
												r[i.ɵLocaleDataIndex.DaysFormat],
												r[i.ɵLocaleDataIndex.DaysStandalone],
											],
											s = $(o, t);
										return $(s, n);
									})(t, o, r)[e.getDay()];
								case q.DayPeriods:
									let l = e.getHours(),
										a = e.getMinutes();
									if (s) {
										let e = (function (e) {
												let t = (0, i.ɵfindLocaleData)(e);
												L(t);
												let n = t[i.ɵLocaleDataIndex.ExtraData][2] || [];
												return n.map((e) =>
													"string" == typeof e ? V(e) : [V(e[0]), V(e[1])],
												);
											})(t),
											n = (function (e, t, n) {
												let r = (0, i.ɵfindLocaleData)(e);
												L(r);
												let o = [
														r[i.ɵLocaleDataIndex.ExtraData][0],
														r[i.ɵLocaleDataIndex.ExtraData][1],
													],
													s = $(o, t) || [];
												return $(s, n) || [];
											})(t, o, r),
											s = e.findIndex((e) => {
												if (Array.isArray(e)) {
													let [t, n] = e,
														r = l >= t.hours && a >= t.minutes,
														o = l < n.hours || (l === n.hours && a < n.minutes);
													if (t.hours < n.hours) {
														if (r && o) return !0;
													} else if (r || o) return !0;
												} else if (e.hours === l && e.minutes === a) return !0;
												return !1;
											});
										if (-1 !== s) return n[s];
									}
									return (function (e, t, n) {
										let r = (0, i.ɵfindLocaleData)(e),
											o = [
												r[i.ɵLocaleDataIndex.DayPeriodsFormat],
												r[i.ɵLocaleDataIndex.DayPeriodsStandalone],
											],
											s = $(o, t);
										return $(s, n);
									})(t, o, r)[l < 12 ? 0 : 1];
								case q.Eras:
									return (function (e, t) {
										let n = (0, i.ɵfindLocaleData)(e),
											r = n[i.ɵLocaleDataIndex.Eras];
										return $(r, t);
									})(t, r)[0 >= e.getFullYear() ? 0 : 1];
								default:
									throw Error(`unexpected translation type ${n}`);
							}
						})(o, s, e, t, n, r);
					};
				}
				function J(e) {
					return function (t, n, r) {
						let o = -1 * r,
							i = F(n, A.MinusSign),
							s = o > 0 ? Math.floor(o / 60) : Math.ceil(o / 60);
						switch (e) {
							case z.Short:
								return (
									(o >= 0 ? "+" : "") + Y(s, 2, i) + Y(Math.abs(o % 60), 2, i)
								);
							case z.ShortGMT:
								return "GMT" + (o >= 0 ? "+" : "") + Y(s, 1, i);
							case z.Long:
								return (
									"GMT" +
									(o >= 0 ? "+" : "") +
									Y(s, 2, i) +
									":" +
									Y(Math.abs(o % 60), 2, i)
								);
							case z.Extended:
								if (0 === r) return "Z";
								return (
									(o >= 0 ? "+" : "") +
									Y(s, 2, i) +
									":" +
									Y(Math.abs(o % 60), 2, i)
								);
							default:
								throw Error(`Unknown zone width "${e}"`);
						}
					};
				}
				function X(e) {
					return G(
						e.getFullYear(),
						e.getMonth(),
						e.getDate() + (4 - e.getDay()),
					);
				}
				function ee(e, t = !1) {
					return function (n, r) {
						let o;
						if (t) {
							let e = new Date(n.getFullYear(), n.getMonth(), 1).getDay() - 1,
								t = n.getDate();
							o = 1 + Math.floor((t + e) / 7);
						} else {
							let e = X(n),
								t = (function (e) {
									let t = G(e, 0, 1).getDay();
									return G(e, 0, 1 + (t <= 4 ? 4 : 11) - t);
								})(e.getFullYear()),
								r = e.getTime() - t.getTime();
							o = 1 + Math.round(r / 6048e5);
						}
						return Y(o, e, F(r, A.MinusSign));
					};
				}
				function et(e, t = !1) {
					return function (n, r) {
						let o = X(n),
							i = o.getFullYear();
						return Y(i, e, F(r, A.MinusSign), t);
					};
				}
				let en = {};
				function er(e, t) {
					e = e.replace(/:/g, "");
					let n = Date.parse("Jan 01, 1970 00:00:00 " + e) / 6e4;
					return isNaN(n) ? t : n;
				}
				function eo(e) {
					return e instanceof Date && !isNaN(e.valueOf());
				}
				let ei = /^(\d+)?\.((\d+)(-(\d+))?)?$/;
				function es(e, t, n, r, o, i, s = !1) {
					let l = "",
						a = !1;
					if (isFinite(e)) {
						let u = (function (e) {
							let t,
								n,
								r,
								o = Math.abs(e) + "",
								i = 0,
								s,
								l;
							for (
								(l = o.indexOf(".")) > -1 && (o = o.replace(".", "")),
									(t = o.search(/e/i)) > 0
										? (l < 0 && (l = t),
										  (l += +o.slice(t + 1)),
										  (o = o.substring(0, t)))
										: l < 0 && (l = o.length),
									t = 0;
								"0" === o.charAt(t);
								t++
							);
							if (t === (r = o.length)) (s = [0]), (l = 1);
							else {
								for (r--; "0" === o.charAt(r); ) r--;
								for (l -= t, s = [], n = 0; t <= r; t++, n++)
									s[n] = Number(o.charAt(t));
							}
							return (
								l > 22 && ((s = s.splice(0, 21)), (i = l - 1), (l = 1)),
								{ digits: s, exponent: i, integerLen: l }
							);
						})(e);
						s &&
							(u = (function (e) {
								if (0 === e.digits[0]) return e;
								let t = e.digits.length - e.integerLen;
								return (
									e.exponent
										? (e.exponent += 2)
										: (0 === t
												? e.digits.push(0, 0)
												: 1 === t && e.digits.push(0),
										  (e.integerLen += 2)),
									e
								);
							})(u));
						let d = t.minInt,
							c = t.minFrac,
							f = t.maxFrac;
						if (i) {
							let e = i.match(ei);
							if (null === e) throw Error(`${i} is not a valid digit info`);
							let t = e[1],
								n = e[3],
								r = e[5];
							null != t && (d = ea(t)),
								null != n && (c = ea(n)),
								null != r ? (f = ea(r)) : null != n && c > f && (f = c);
						}
						(function (e, t, n) {
							if (t > n)
								throw Error(
									`The minimum number of digits after fraction (${t}) is higher than the maximum (${n}).`,
								);
							let r = e.digits,
								o = r.length - e.integerLen,
								i = Math.min(Math.max(t, o), n),
								s = i + e.integerLen,
								l = r[s];
							if (s > 0) {
								r.splice(Math.max(e.integerLen, s));
								for (let e = s; e < r.length; e++) r[e] = 0;
							} else {
								(o = Math.max(0, o)),
									(e.integerLen = 1),
									(r.length = Math.max(1, (s = i + 1))),
									(r[0] = 0);
								for (let e = 1; e < s; e++) r[e] = 0;
							}
							if (l >= 5) {
								if (s - 1 < 0) {
									for (let t = 0; t > s; t--) r.unshift(0), e.integerLen++;
									r.unshift(1), e.integerLen++;
								} else r[s - 1]++;
							}
							for (; o < Math.max(0, i); o++) r.push(0);
							let a = 0 !== i,
								u = t + e.integerLen,
								d = r.reduceRight(function (e, t, n, r) {
									return (
										(t += e),
										(r[n] = t < 10 ? t : t - 10),
										a && (0 === r[n] && n >= u ? r.pop() : (a = !1)),
										t >= 10 ? 1 : 0
									);
								}, 0);
							d && (r.unshift(d), e.integerLen++);
						})(u, c, f);
						let h = u.digits,
							p = u.integerLen,
							m = u.exponent,
							g = [];
						for (a = h.every((e) => !e); p < d; p++) h.unshift(0);
						for (; p < 0; p++) h.unshift(0);
						p > 0 ? (g = h.splice(p, h.length)) : ((g = h), (h = [0]));
						let v = [];
						for (
							h.length >= t.lgSize &&
							v.unshift(h.splice(-t.lgSize, h.length).join(""));
							h.length > t.gSize;
						)
							v.unshift(h.splice(-t.gSize, h.length).join(""));
						h.length && v.unshift(h.join("")),
							(l = v.join(F(n, r))),
							g.length && (l += F(n, o) + g.join("")),
							m && (l += F(n, A.Exponential) + "+" + m);
					} else l = F(n, A.Infinity);
					return (l =
						e < 0 && !a ? t.negPre + l + t.negSuf : t.posPre + l + t.posSuf);
				}
				function el(e, t = "-") {
					let n = {
							minInt: 1,
							minFrac: 0,
							maxFrac: 0,
							posPre: "",
							posSuf: "",
							negPre: "",
							negSuf: "",
							gSize: 0,
							lgSize: 0,
						},
						r = e.split(";"),
						o = r[0],
						i = r[1],
						s =
							-1 !== o.indexOf(".")
								? o.split(".")
								: [
										o.substring(0, o.lastIndexOf("0") + 1),
										o.substring(o.lastIndexOf("0") + 1),
								  ],
						l = s[0],
						a = s[1] || "";
					n.posPre = l.substring(0, l.indexOf("#"));
					for (let e = 0; e < a.length; e++) {
						let t = a.charAt(e);
						"0" === t
							? (n.minFrac = n.maxFrac = e + 1)
							: "#" === t
							? (n.maxFrac = e + 1)
							: (n.posSuf += t);
					}
					let u = l.split(",");
					if (
						((n.gSize = u[1] ? u[1].length : 0),
						(n.lgSize = u[2] || u[1] ? (u[2] || u[1]).length : 0),
						i)
					) {
						let e = o.length - n.posPre.length - n.posSuf.length,
							t = i.indexOf("#");
						(n.negPre = i.substring(0, t).replace(/'/g, "")),
							(n.negSuf = i.slice(t + e).replace(/'/g, ""));
					} else (n.negPre = t + n.posPre), (n.negSuf = n.posSuf);
					return n;
				}
				function ea(e) {
					let t = parseInt(e);
					if (isNaN(t))
						throw Error("Invalid integer literal when parsing " + e);
					return t;
				}
				let eu = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function (e) {
								let t = null;
								if (e) t = new e();
								else {
									var n;
									(n = i.ɵɵinject(i.LOCALE_ID)), (t = new ec(n));
								}
								return t;
							},
							providedIn: "root",
						})),
						e
					);
				})();
				function ed(e, t, n, r) {
					let o = `=${e}`;
					if (t.indexOf(o) > -1) return o;
					if (((o = n.getPluralCategory(e, r)), t.indexOf(o) > -1)) return o;
					if (t.indexOf("other") > -1) return "other";
					throw Error(`No plural message found for value "${e}"`);
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let ec = (() => {
					class e extends eu {
						getPluralCategory(e, t) {
							let n = N(t || this.locale)(e);
							switch (n) {
								case C.Zero:
									return "zero";
								case C.One:
									return "one";
								case C.Two:
									return "two";
								case C.Few:
									return "few";
								case C.Many:
									return "many";
								default:
									return "other";
							}
						}
						constructor(e) {
							super(), (this.locale = e);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵinject(i.LOCALE_ID));
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				function ef(e, t) {
					for (let n of ((t = encodeURIComponent(t)), e.split(";"))) {
						let e = n.indexOf("="),
							[r, o] = -1 == e ? [n, ""] : [n.slice(0, e), n.slice(e + 1)];
						if (r.trim() === t) return decodeURIComponent(o);
					}
					return null;
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let eh = /\s+/,
					ep = [];
				function em(e) {
					let t = e.get(i.NgModuleRef);
					return t.injector;
				}
				(() => {
					class e {
						set klass(e) {
							this.initialClasses = null != e ? e.trim().split(eh) : ep;
						}
						set ngClass(e) {
							this.rawClass = "string" == typeof e ? e.trim().split(eh) : e;
						}
						ngDoCheck() {
							for (let e of this.initialClasses) this._updateState(e, !0);
							let e = this.rawClass;
							if (Array.isArray(e) || e instanceof Set)
								for (let t of e) this._updateState(t, !0);
							else if (null != e)
								for (let t of Object.keys(e)) this._updateState(t, !!e[t]);
							this._applyStateDiff();
						}
						_updateState(e, t) {
							let n = this.stateMap.get(e);
							void 0 !== n
								? (n.enabled !== t && ((n.changed = !0), (n.enabled = t)),
								  (n.touched = !0))
								: this.stateMap.set(e, {
										enabled: t,
										changed: !0,
										touched: !0,
								  });
						}
						_applyStateDiff() {
							for (let e of this.stateMap) {
								let t = e[0],
									n = e[1];
								n.changed
									? (this._toggleClass(t, n.enabled), (n.changed = !1))
									: !n.touched &&
									  (n.enabled && this._toggleClass(t, !1),
									  this.stateMap.delete(t)),
									(n.touched = !1);
							}
						}
						_toggleClass(e, t) {
							if (ngDevMode && "string" != typeof e)
								throw Error(
									`NgClass can only toggle CSS classes expressed as strings, got ${(0,
									i.ɵstringify)(e)}`,
								);
							(e = e.trim()).length > 0 &&
								e.split(eh).forEach((e) => {
									t
										? this._renderer.addClass(this._ngEl.nativeElement, e)
										: this._renderer.removeClass(this._ngEl.nativeElement, e);
								});
						}
						constructor(e, t, n, r) {
							(this._iterableDiffers = e),
								(this._keyValueDiffers = t),
								(this._ngEl = n),
								(this._renderer = r),
								(this.initialClasses = ep),
								(this.stateMap = new Map());
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(
							i.ɵɵdirectiveInject(i.IterableDiffers),
							i.ɵɵdirectiveInject(i.KeyValueDiffers),
							i.ɵɵdirectiveInject(i.ElementRef),
							i.ɵɵdirectiveInject(i.Renderer2),
						);
					}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngClass", ""]],
							inputs: { klass: ["class", "klass"], ngClass: "ngClass" },
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							ngOnChanges(e) {
								let {
									_viewContainerRef: t,
									ngComponentOutletNgModule: n,
									ngComponentOutletNgModuleFactory: r,
								} = this;
								if (
									(t.clear(),
									(this._componentRef = void 0),
									this.ngComponentOutlet)
								) {
									let o = this.ngComponentOutletInjector || t.parentInjector;
									(e.ngComponentOutletNgModule ||
										e.ngComponentOutletNgModuleFactory) &&
										(this._moduleRef && this._moduleRef.destroy(),
										n
											? (this._moduleRef = (0, i.createNgModule)(n, em(o)))
											: r
											? (this._moduleRef = r.create(em(o)))
											: (this._moduleRef = void 0)),
										(this._componentRef = t.createComponent(
											this.ngComponentOutlet,
											{
												index: t.length,
												injector: o,
												ngModuleRef: this._moduleRef,
												projectableNodes: this.ngComponentOutletContent,
											},
										));
								}
							}
							ngOnDestroy() {
								this._moduleRef && this._moduleRef.destroy();
							}
							constructor(e) {
								(this._viewContainerRef = e), (this.ngComponentOutlet = null);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵdirectiveInject(i.ViewContainerRef));
						}),
							(e.ɵdir = i.ɵɵdefineDirective({
								type: e,
								selectors: [["", "ngComponentOutlet", ""]],
								inputs: {
									ngComponentOutlet: "ngComponentOutlet",
									ngComponentOutletInjector: "ngComponentOutletInjector",
									ngComponentOutletContent: "ngComponentOutletContent",
									ngComponentOutletNgModule: "ngComponentOutletNgModule",
									ngComponentOutletNgModuleFactory:
										"ngComponentOutletNgModuleFactory",
								},
								standalone: !0,
								features: [i.ɵɵNgOnChangesFeature],
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				class eg {
					get first() {
						return 0 === this.index;
					}
					get last() {
						return this.index === this.count - 1;
					}
					get even() {
						return this.index % 2 == 0;
					}
					get odd() {
						return !this.even;
					}
					constructor(e, t, n, r) {
						(this.$implicit = e),
							(this.ngForOf = t),
							(this.index = n),
							(this.count = r);
					}
				}
				function ev(e, t) {
					e.context.$implicit = t.item;
				}
				(() => {
					class e {
						set ngForOf(e) {
							(this._ngForOf = e), (this._ngForOfDirty = !0);
						}
						set ngForTrackBy(e) {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								null != e &&
								"function" != typeof e &&
								console.warn(
									`trackBy must be a function, but received ${JSON.stringify(
										e,
									)}. See https://angular.io/api/common/NgForOf#change-propagation for more information.`,
								),
								(this._trackByFn = e);
						}
						get ngForTrackBy() {
							return this._trackByFn;
						}
						set ngForTemplate(e) {
							e && (this._template = e);
						}
						ngDoCheck() {
							if (this._ngForOfDirty) {
								this._ngForOfDirty = !1;
								let e = this._ngForOf;
								if (!this._differ && e) {
									if ("undefined" == typeof ngDevMode || ngDevMode)
										try {
											this._differ = this._differs
												.find(e)
												.create(this.ngForTrackBy);
										} catch (n) {
											let t = `Cannot find a differ supporting object '${e}' of type '${(function (
												e,
											) {
												return e.name || typeof e;
											})(
												e,
											)}'. NgFor only supports binding to Iterables, such as Arrays.`;
											throw (
												("object" == typeof e &&
													(t += " Did you mean to use the keyvalue pipe?"),
												new i.ɵRuntimeError(-2200, t))
											);
										}
									else
										this._differ = this._differs
											.find(e)
											.create(this.ngForTrackBy);
								}
							}
							if (this._differ) {
								let e = this._differ.diff(this._ngForOf);
								e && this._applyChanges(e);
							}
						}
						_applyChanges(e) {
							let t = this._viewContainer;
							e.forEachOperation((e, n, r) => {
								if (null == e.previousIndex)
									t.createEmbeddedView(
										this._template,
										new eg(e.item, this._ngForOf, -1, -1),
										null === r ? void 0 : r,
									);
								else if (null == r) t.remove(null === n ? void 0 : n);
								else if (null !== n) {
									let o = t.get(n);
									t.move(o, r), ev(o, e);
								}
							});
							for (let e = 0, n = t.length; e < n; e++) {
								let r = t.get(e),
									o = r.context;
								(o.index = e), (o.count = n), (o.ngForOf = this._ngForOf);
							}
							e.forEachIdentityChange((e) => {
								let n = t.get(e.currentIndex);
								ev(n, e);
							});
						}
						static ngTemplateContextGuard(e, t) {
							return !0;
						}
						constructor(e, t, n) {
							(this._viewContainer = e),
								(this._template = t),
								(this._differs = n),
								(this._ngForOf = null),
								(this._ngForOfDirty = !0),
								(this._differ = null);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(
							i.ɵɵdirectiveInject(i.ViewContainerRef),
							i.ɵɵdirectiveInject(i.TemplateRef),
							i.ɵɵdirectiveInject(i.IterableDiffers),
						);
					}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngFor", "", "ngForOf", ""]],
							inputs: {
								ngForOf: "ngForOf",
								ngForTrackBy: "ngForTrackBy",
								ngForTemplate: "ngForTemplate",
							},
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				(() => {
					class e {
						set ngIf(e) {
							(this._context.$implicit = this._context.ngIf = e),
								this._updateView();
						}
						set ngIfThen(e) {
							eb("ngIfThen", e),
								(this._thenTemplateRef = e),
								(this._thenViewRef = null),
								this._updateView();
						}
						set ngIfElse(e) {
							eb("ngIfElse", e),
								(this._elseTemplateRef = e),
								(this._elseViewRef = null),
								this._updateView();
						}
						_updateView() {
							this._context.$implicit
								? !this._thenViewRef &&
								  (this._viewContainer.clear(),
								  (this._elseViewRef = null),
								  this._thenTemplateRef &&
										(this._thenViewRef = this._viewContainer.createEmbeddedView(
											this._thenTemplateRef,
											this._context,
										)))
								: !this._elseViewRef &&
								  (this._viewContainer.clear(),
								  (this._thenViewRef = null),
								  this._elseTemplateRef &&
										(this._elseViewRef = this._viewContainer.createEmbeddedView(
											this._elseTemplateRef,
											this._context,
										)));
						}
						static ngTemplateContextGuard(e, t) {
							return !0;
						}
						constructor(e, t) {
							(this._viewContainer = e),
								(this._context = new ey()),
								(this._thenTemplateRef = null),
								(this._elseTemplateRef = null),
								(this._thenViewRef = null),
								(this._elseViewRef = null),
								(this._thenTemplateRef = t);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(
							i.ɵɵdirectiveInject(i.ViewContainerRef),
							i.ɵɵdirectiveInject(i.TemplateRef),
						);
					}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngIf", ""]],
							inputs: {
								ngIf: "ngIf",
								ngIfThen: "ngIfThen",
								ngIfElse: "ngIfElse",
							},
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				class ey {
					constructor() {
						(this.$implicit = null), (this.ngIf = null);
					}
				}
				function eb(e, t) {
					let n = !!(!t || t.createEmbeddedView);
					if (!n)
						throw Error(
							`${e} must be a TemplateRef, but received '${(0, i.ɵstringify)(
								t,
							)}'.`,
						);
				}
				class e_ {
					create() {
						(this._created = !0),
							this._viewContainerRef.createEmbeddedView(this._templateRef);
					}
					destroy() {
						(this._created = !1), this._viewContainerRef.clear();
					}
					enforceState(e) {
						e && !this._created
							? this.create()
							: !e && this._created && this.destroy();
					}
					constructor(e, t) {
						(this._viewContainerRef = e),
							(this._templateRef = t),
							(this._created = !1);
					}
				}
				let ej = (() => {
					class e {
						set ngSwitch(e) {
							(this._ngSwitch = e),
								0 === this._caseCount && this._updateDefaultCases(!0);
						}
						_addCase() {
							return this._caseCount++;
						}
						_addDefault(e) {
							this._defaultViews.push(e);
						}
						_matchCase(e) {
							let t = e == this._ngSwitch;
							return (
								(this._lastCasesMatched = this._lastCasesMatched || t),
								this._lastCaseCheckIndex++,
								this._lastCaseCheckIndex === this._caseCount &&
									(this._updateDefaultCases(!this._lastCasesMatched),
									(this._lastCaseCheckIndex = 0),
									(this._lastCasesMatched = !1)),
								t
							);
						}
						_updateDefaultCases(e) {
							if (this._defaultViews.length > 0 && e !== this._defaultUsed)
								for (let t of ((this._defaultUsed = e), this._defaultViews))
									t.enforceState(e);
						}
						constructor() {
							(this._defaultViews = []),
								(this._defaultUsed = !1),
								(this._caseCount = 0),
								(this._lastCaseCheckIndex = 0),
								(this._lastCasesMatched = !1);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngSwitch", ""]],
							inputs: { ngSwitch: "ngSwitch" },
							standalone: !0,
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let ex = (() => {
					class e {
						ngDoCheck() {
							this._view.enforceState(
								this.ngSwitch._matchCase(this.ngSwitchCase),
							);
						}
						constructor(e, t, n) {
							(this.ngSwitch = n),
								("undefined" == typeof ngDevMode || ngDevMode) &&
									!n &&
									ew("ngSwitchCase", "NgSwitchCase"),
								n._addCase(),
								(this._view = new e_(e, t));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(i.ViewContainerRef),
								i.ɵɵdirectiveInject(i.TemplateRef),
								i.ɵɵdirectiveInject(ej, 9),
							);
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngSwitchCase", ""]],
							inputs: { ngSwitchCase: "ngSwitchCase" },
							standalone: !0,
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let eD = (() => {
					class e {
						constructor(e, t, n) {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								!n &&
								ew("ngSwitchDefault", "NgSwitchDefault"),
								n._addDefault(new e_(e, t));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(i.ViewContainerRef),
								i.ɵɵdirectiveInject(i.TemplateRef),
								i.ɵɵdirectiveInject(ej, 9),
							);
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngSwitchDefault", ""]],
							standalone: !0,
						})),
						e
					);
				})();
				function ew(e, t) {
					throw new i.ɵRuntimeError(
						2e3,
						`An element with the "${e}" attribute (matching the "${t}" directive) must be located inside an element with the "ngSwitch" attribute (matching "NgSwitch" directive)`,
					);
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let eM = (() => {
					class e {
						set ngPlural(e) {
							this._updateView(e);
						}
						addCase(e, t) {
							this._caseViews[e] = t;
						}
						_updateView(e) {
							this._clearViews();
							let t = Object.keys(this._caseViews),
								n = ed(e, t, this._localization);
							this._activateView(this._caseViews[n]);
						}
						_clearViews() {
							this._activeView && this._activeView.destroy();
						}
						_activateView(e) {
							e && ((this._activeView = e), this._activeView.create());
						}
						constructor(e) {
							(this._localization = e), (this._caseViews = {});
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵdirectiveInject(eu));
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "ngPlural", ""]],
							inputs: { ngPlural: "ngPlural" },
							standalone: !0,
						})),
						e
					);
				})();
				function eC(e, t) {
					return new i.ɵRuntimeError(
						2100,
						ngDevMode &&
							`InvalidPipeArgument: '${t}' for pipe '${(0, i.ɵstringify)(e)}'`,
					);
				}
				"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							constructor(e, t, n, r) {
								this.value = e;
								let o = !isNaN(Number(e));
								r.addCase(o ? `=${e}` : e, new e_(n, t));
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵinjectAttribute("ngPluralCase"),
								i.ɵɵdirectiveInject(i.TemplateRef),
								i.ɵɵdirectiveInject(i.ViewContainerRef),
								i.ɵɵdirectiveInject(eM, 1),
							);
						}),
							(e.ɵdir = i.ɵɵdefineDirective({
								type: e,
								selectors: [["", "ngPluralCase", ""]],
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							set ngStyle(e) {
								(this._ngStyle = e),
									!this._differ &&
										e &&
										(this._differ = this._differs.find(e).create());
							}
							ngDoCheck() {
								if (this._differ) {
									let e = this._differ.diff(this._ngStyle);
									e && this._applyChanges(e);
								}
							}
							_setStyle(e, t) {
								let [n, r] = e.split("."),
									o =
										-1 === n.indexOf("-")
											? void 0
											: i.RendererStyleFlags2.DashCase;
								null != t
									? this._renderer.setStyle(
											this._ngEl.nativeElement,
											n,
											r ? `${t}${r}` : t,
											o,
									  )
									: this._renderer.removeStyle(this._ngEl.nativeElement, n, o);
							}
							_applyChanges(e) {
								e.forEachRemovedItem((e) => this._setStyle(e.key, null)),
									e.forEachAddedItem((e) =>
										this._setStyle(e.key, e.currentValue),
									),
									e.forEachChangedItem((e) =>
										this._setStyle(e.key, e.currentValue),
									);
							}
							constructor(e, t, n) {
								(this._ngEl = e),
									(this._differs = t),
									(this._renderer = n),
									(this._ngStyle = null),
									(this._differ = null);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(i.ElementRef),
								i.ɵɵdirectiveInject(i.KeyValueDiffers),
								i.ɵɵdirectiveInject(i.Renderer2),
							);
						}),
							(e.ɵdir = i.ɵɵdefineDirective({
								type: e,
								selectors: [["", "ngStyle", ""]],
								inputs: { ngStyle: "ngStyle" },
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							ngOnChanges(e) {
								if (e.ngTemplateOutlet || e.ngTemplateOutletInjector) {
									let e = this._viewContainerRef;
									if (
										(this._viewRef && e.remove(e.indexOf(this._viewRef)),
										this.ngTemplateOutlet)
									) {
										let {
											ngTemplateOutlet: t,
											ngTemplateOutletContext: n,
											ngTemplateOutletInjector: r,
										} = this;
										this._viewRef = e.createEmbeddedView(
											t,
											n,
											r ? { injector: r } : void 0,
										);
									} else this._viewRef = null;
								} else
									this._viewRef &&
										e.ngTemplateOutletContext &&
										this.ngTemplateOutletContext &&
										(this._viewRef.context = this.ngTemplateOutletContext);
							}
							constructor(e) {
								(this._viewContainerRef = e),
									(this._viewRef = null),
									(this.ngTemplateOutletContext = null),
									(this.ngTemplateOutlet = null),
									(this.ngTemplateOutletInjector = null);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵdirectiveInject(i.ViewContainerRef));
						}),
							(e.ɵdir = i.ɵɵdefineDirective({
								type: e,
								selectors: [["", "ngTemplateOutlet", ""]],
								inputs: {
									ngTemplateOutletContext: "ngTemplateOutletContext",
									ngTemplateOutlet: "ngTemplateOutlet",
									ngTemplateOutletInjector: "ngTemplateOutletInjector",
								},
								standalone: !0,
								features: [i.ɵɵNgOnChangesFeature],
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let eS = new (class e {
						createSubscription(e, t) {
							return e.then(t, (e) => {
								throw e;
							});
						}
						dispose(e) {}
					})(),
					eE = new (class e {
						createSubscription(e, t) {
							return e.subscribe({
								next: t,
								error: (e) => {
									throw e;
								},
							});
						}
						dispose(e) {
							e.unsubscribe();
						}
					})();
				(() => {
					class e {
						ngOnDestroy() {
							this._subscription && this._dispose(), (this._ref = null);
						}
						transform(e) {
							return this._obj
								? e !== this._obj
									? (this._dispose(), this.transform(e))
									: this._latestValue
								: (e && this._subscribe(e), this._latestValue);
						}
						_subscribe(e) {
							(this._obj = e),
								(this._strategy = this._selectStrategy(e)),
								(this._subscription = this._strategy.createSubscription(
									e,
									(t) => this._updateLatestValue(e, t),
								));
						}
						_selectStrategy(t) {
							if ((0, i.ɵisPromise)(t)) return eS;
							if ((0, i.ɵisSubscribable)(t)) return eE;
							throw eC(e, t);
						}
						_dispose() {
							this._strategy.dispose(this._subscription),
								(this._latestValue = null),
								(this._subscription = null),
								(this._obj = null);
						}
						_updateLatestValue(e, t) {
							e === this._obj &&
								((this._latestValue = t), this._ref.markForCheck());
						}
						constructor(e) {
							(this._latestValue = null),
								(this._subscription = null),
								(this._obj = null),
								(this._strategy = null),
								(this._ref = e);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(i.ɵɵdirectiveInject(i.ChangeDetectorRef, 16));
					}),
						(e.ɵpipe = i.ɵɵdefinePipe({
							name: "async",
							type: e,
							pure: !1,
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t) {
								if (null == t) return null;
								if ("string" != typeof t) throw eC(e, t);
								return t.toLowerCase();
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "lowercase",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let eO =
					/(?:[0-9A-Za-z\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0370-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u048A-\u052F\u0531-\u0556\u0559\u0560-\u0588\u05D0-\u05EA\u05EF-\u05F2\u0620-\u064A\u066E\u066F\u0671-\u06D3\u06D5\u06E5\u06E6\u06EE\u06EF\u06FA-\u06FC\u06FF\u0710\u0712-\u072F\u074D-\u07A5\u07B1\u07CA-\u07EA\u07F4\u07F5\u07FA\u0800-\u0815\u081A\u0824\u0828\u0840-\u0858\u0860-\u086A\u0870-\u0887\u0889-\u088E\u08A0-\u08C9\u0904-\u0939\u093D\u0950\u0958-\u0961\u0971-\u0980\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BD\u09CE\u09DC\u09DD\u09DF-\u09E1\u09F0\u09F1\u09FC\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A59-\u0A5C\u0A5E\u0A72-\u0A74\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABD\u0AD0\u0AE0\u0AE1\u0AF9\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3D\u0B5C\u0B5D\u0B5F-\u0B61\u0B71\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BD0\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D\u0C58-\u0C5A\u0C5D\u0C60\u0C61\u0C80\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBD\u0CDD\u0CDE\u0CE0\u0CE1\u0CF1\u0CF2\u0D04-\u0D0C\u0D0E-\u0D10\u0D12-\u0D3A\u0D3D\u0D4E\u0D54-\u0D56\u0D5F-\u0D61\u0D7A-\u0D7F\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0E01-\u0E30\u0E32\u0E33\u0E40-\u0E46\u0E81\u0E82\u0E84\u0E86-\u0E8A\u0E8C-\u0EA3\u0EA5\u0EA7-\u0EB0\u0EB2\u0EB3\u0EBD\u0EC0-\u0EC4\u0EC6\u0EDC-\u0EDF\u0F00\u0F40-\u0F47\u0F49-\u0F6C\u0F88-\u0F8C\u1000-\u102A\u103F\u1050-\u1055\u105A-\u105D\u1061\u1065\u1066\u106E-\u1070\u1075-\u1081\u108E\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u1380-\u138F\u13A0-\u13F5\u13F8-\u13FD\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16F1-\u16F8\u1700-\u1711\u171F-\u1731\u1740-\u1751\u1760-\u176C\u176E-\u1770\u1780-\u17B3\u17D7\u17DC\u1820-\u1878\u1880-\u1884\u1887-\u18A8\u18AA\u18B0-\u18F5\u1900-\u191E\u1950-\u196D\u1970-\u1974\u1980-\u19AB\u19B0-\u19C9\u1A00-\u1A16\u1A20-\u1A54\u1AA7\u1B05-\u1B33\u1B45-\u1B4C\u1B83-\u1BA0\u1BAE\u1BAF\u1BBA-\u1BE5\u1C00-\u1C23\u1C4D-\u1C4F\u1C5A-\u1C7D\u1C80-\u1C88\u1C90-\u1CBA\u1CBD-\u1CBF\u1CE9-\u1CEC\u1CEE-\u1CF3\u1CF5\u1CF6\u1CFA\u1D00-\u1DBF\u1E00-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u2071\u207F\u2090-\u209C\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2183\u2184\u2C00-\u2CE4\u2CEB-\u2CEE\u2CF2\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D80-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2E2F\u3005\u3006\u3031-\u3035\u303B\u303C\u3041-\u3096\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312F\u3131-\u318E\u31A0-\u31BF\u31F0-\u31FF\u3400-\u4DBF\u4E00-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA61F\uA62A\uA62B\uA640-\uA66E\uA67F-\uA69D\uA6A0-\uA6E5\uA717-\uA71F\uA722-\uA788\uA78B-\uA7CA\uA7D0\uA7D1\uA7D3\uA7D5-\uA7D9\uA7F2-\uA801\uA803-\uA805\uA807-\uA80A\uA80C-\uA822\uA840-\uA873\uA882-\uA8B3\uA8F2-\uA8F7\uA8FB\uA8FD\uA8FE\uA90A-\uA925\uA930-\uA946\uA960-\uA97C\uA984-\uA9B2\uA9CF\uA9E0-\uA9E4\uA9E6-\uA9EF\uA9FA-\uA9FE\uAA00-\uAA28\uAA40-\uAA42\uAA44-\uAA4B\uAA60-\uAA76\uAA7A\uAA7E-\uAAAF\uAAB1\uAAB5\uAAB6\uAAB9-\uAABD\uAAC0\uAAC2\uAADB-\uAADD\uAAE0-\uAAEA\uAAF2-\uAAF4\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB69\uAB70-\uABE2\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D\uFB1F-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE70-\uFE74\uFE76-\uFEFC\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC]|\uD800[\uDC00-\uDC0B\uDC0D-\uDC26\uDC28-\uDC3A\uDC3C\uDC3D\uDC3F-\uDC4D\uDC50-\uDC5D\uDC80-\uDCFA\uDE80-\uDE9C\uDEA0-\uDED0\uDF00-\uDF1F\uDF2D-\uDF40\uDF42-\uDF49\uDF50-\uDF75\uDF80-\uDF9D\uDFA0-\uDFC3\uDFC8-\uDFCF]|\uD801[\uDC00-\uDC9D\uDCB0-\uDCD3\uDCD8-\uDCFB\uDD00-\uDD27\uDD30-\uDD63\uDD70-\uDD7A\uDD7C-\uDD8A\uDD8C-\uDD92\uDD94\uDD95\uDD97-\uDDA1\uDDA3-\uDDB1\uDDB3-\uDDB9\uDDBB\uDDBC\uDE00-\uDF36\uDF40-\uDF55\uDF60-\uDF67\uDF80-\uDF85\uDF87-\uDFB0\uDFB2-\uDFBA]|\uD802[\uDC00-\uDC05\uDC08\uDC0A-\uDC35\uDC37\uDC38\uDC3C\uDC3F-\uDC55\uDC60-\uDC76\uDC80-\uDC9E\uDCE0-\uDCF2\uDCF4\uDCF5\uDD00-\uDD15\uDD20-\uDD39\uDD80-\uDDB7\uDDBE\uDDBF\uDE00\uDE10-\uDE13\uDE15-\uDE17\uDE19-\uDE35\uDE60-\uDE7C\uDE80-\uDE9C\uDEC0-\uDEC7\uDEC9-\uDEE4\uDF00-\uDF35\uDF40-\uDF55\uDF60-\uDF72\uDF80-\uDF91]|\uD803[\uDC00-\uDC48\uDC80-\uDCB2\uDCC0-\uDCF2\uDD00-\uDD23\uDE80-\uDEA9\uDEB0\uDEB1\uDF00-\uDF1C\uDF27\uDF30-\uDF45\uDF70-\uDF81\uDFB0-\uDFC4\uDFE0-\uDFF6]|\uD804[\uDC03-\uDC37\uDC71\uDC72\uDC75\uDC83-\uDCAF\uDCD0-\uDCE8\uDD03-\uDD26\uDD44\uDD47\uDD50-\uDD72\uDD76\uDD83-\uDDB2\uDDC1-\uDDC4\uDDDA\uDDDC\uDE00-\uDE11\uDE13-\uDE2B\uDE80-\uDE86\uDE88\uDE8A-\uDE8D\uDE8F-\uDE9D\uDE9F-\uDEA8\uDEB0-\uDEDE\uDF05-\uDF0C\uDF0F\uDF10\uDF13-\uDF28\uDF2A-\uDF30\uDF32\uDF33\uDF35-\uDF39\uDF3D\uDF50\uDF5D-\uDF61]|\uD805[\uDC00-\uDC34\uDC47-\uDC4A\uDC5F-\uDC61\uDC80-\uDCAF\uDCC4\uDCC5\uDCC7\uDD80-\uDDAE\uDDD8-\uDDDB\uDE00-\uDE2F\uDE44\uDE80-\uDEAA\uDEB8\uDF00-\uDF1A\uDF40-\uDF46]|\uD806[\uDC00-\uDC2B\uDCA0-\uDCDF\uDCFF-\uDD06\uDD09\uDD0C-\uDD13\uDD15\uDD16\uDD18-\uDD2F\uDD3F\uDD41\uDDA0-\uDDA7\uDDAA-\uDDD0\uDDE1\uDDE3\uDE00\uDE0B-\uDE32\uDE3A\uDE50\uDE5C-\uDE89\uDE9D\uDEB0-\uDEF8]|\uD807[\uDC00-\uDC08\uDC0A-\uDC2E\uDC40\uDC72-\uDC8F\uDD00-\uDD06\uDD08\uDD09\uDD0B-\uDD30\uDD46\uDD60-\uDD65\uDD67\uDD68\uDD6A-\uDD89\uDD98\uDEE0-\uDEF2\uDFB0]|\uD808[\uDC00-\uDF99]|\uD809[\uDC80-\uDD43]|\uD80B[\uDF90-\uDFF0]|[\uD80C\uD81C-\uD820\uD822\uD840-\uD868\uD86A-\uD86C\uD86F-\uD872\uD874-\uD879\uD880-\uD883][\uDC00-\uDFFF]|\uD80D[\uDC00-\uDC2E]|\uD811[\uDC00-\uDE46]|\uD81A[\uDC00-\uDE38\uDE40-\uDE5E\uDE70-\uDEBE\uDED0-\uDEED\uDF00-\uDF2F\uDF40-\uDF43\uDF63-\uDF77\uDF7D-\uDF8F]|\uD81B[\uDE40-\uDE7F\uDF00-\uDF4A\uDF50\uDF93-\uDF9F\uDFE0\uDFE1\uDFE3]|\uD821[\uDC00-\uDFF7]|\uD823[\uDC00-\uDCD5\uDD00-\uDD08]|\uD82B[\uDFF0-\uDFF3\uDFF5-\uDFFB\uDFFD\uDFFE]|\uD82C[\uDC00-\uDD22\uDD50-\uDD52\uDD64-\uDD67\uDD70-\uDEFB]|\uD82F[\uDC00-\uDC6A\uDC70-\uDC7C\uDC80-\uDC88\uDC90-\uDC99]|\uD835[\uDC00-\uDC54\uDC56-\uDC9C\uDC9E\uDC9F\uDCA2\uDCA5\uDCA6\uDCA9-\uDCAC\uDCAE-\uDCB9\uDCBB\uDCBD-\uDCC3\uDCC5-\uDD05\uDD07-\uDD0A\uDD0D-\uDD14\uDD16-\uDD1C\uDD1E-\uDD39\uDD3B-\uDD3E\uDD40-\uDD44\uDD46\uDD4A-\uDD50\uDD52-\uDEA5\uDEA8-\uDEC0\uDEC2-\uDEDA\uDEDC-\uDEFA\uDEFC-\uDF14\uDF16-\uDF34\uDF36-\uDF4E\uDF50-\uDF6E\uDF70-\uDF88\uDF8A-\uDFA8\uDFAA-\uDFC2\uDFC4-\uDFCB]|\uD837[\uDF00-\uDF1E]|\uD838[\uDD00-\uDD2C\uDD37-\uDD3D\uDD4E\uDE90-\uDEAD\uDEC0-\uDEEB]|\uD839[\uDFE0-\uDFE6\uDFE8-\uDFEB\uDFED\uDFEE\uDFF0-\uDFFE]|\uD83A[\uDC00-\uDCC4\uDD00-\uDD43\uDD4B]|\uD83B[\uDE00-\uDE03\uDE05-\uDE1F\uDE21\uDE22\uDE24\uDE27\uDE29-\uDE32\uDE34-\uDE37\uDE39\uDE3B\uDE42\uDE47\uDE49\uDE4B\uDE4D-\uDE4F\uDE51\uDE52\uDE54\uDE57\uDE59\uDE5B\uDE5D\uDE5F\uDE61\uDE62\uDE64\uDE67-\uDE6A\uDE6C-\uDE72\uDE74-\uDE77\uDE79-\uDE7C\uDE7E\uDE80-\uDE89\uDE8B-\uDE9B\uDEA1-\uDEA3\uDEA5-\uDEA9\uDEAB-\uDEBB]|\uD869[\uDC00-\uDEDF\uDF00-\uDFFF]|\uD86D[\uDC00-\uDF38\uDF40-\uDFFF]|\uD86E[\uDC00-\uDC1D\uDC20-\uDFFF]|\uD873[\uDC00-\uDEA1\uDEB0-\uDFFF]|\uD87A[\uDC00-\uDFE0]|\uD87E[\uDC00-\uDE1D]|\uD884[\uDC00-\uDF4A])\S*/g;
				(() => {
					class e {
						transform(t) {
							if (null == t) return null;
							if ("string" != typeof t) throw eC(e, t);
							return t.replace(
								eO,
								(e) => e[0].toUpperCase() + e.slice(1).toLowerCase(),
							);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)();
					}),
						(e.ɵpipe = i.ɵɵdefinePipe({
							name: "titlecase",
							type: e,
							pure: !0,
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t) {
								if (null == t) return null;
								if ("string" != typeof t) throw eC(e, t);
								return t.toUpperCase();
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "uppercase",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let eA = new i.InjectionToken("DATE_PIPE_DEFAULT_TIMEZONE"),
					eI = new i.InjectionToken("DATE_PIPE_DEFAULT_OPTIONS");
				(() => {
					class e {
						transform(t, n, r, o) {
							if (null == t || "" === t || t != t) return null;
							try {
								var s, l, a, u, d;
								let e =
										null !==
											(a =
												null != n
													? n
													: null === (s = this.defaultOptions) || void 0 === s
													? void 0
													: s.dateFormat) && void 0 !== a
											? a
											: "mediumDate",
									c =
										null !==
											(d =
												null !==
													(u =
														null != r
															? r
															: null === (l = this.defaultOptions) ||
															  void 0 === l
															? void 0
															: l.timezone) && void 0 !== u
													? u
													: this.defaultTimezone) && void 0 !== d
											? d
											: void 0;
								return (function (e, t, n, r) {
									let o,
										s = (function (e) {
											if (eo(e)) return e;
											if ("number" == typeof e && !isNaN(e)) return new Date(e);
											if ("string" == typeof e) {
												let t;
												if (
													((e = e.trim()),
													/^(\d{4}(-\d{1,2}(-\d{1,2})?)?)$/.test(e))
												) {
													let [t, n = 1, r = 1] = e.split("-").map((e) => +e);
													return G(t, n - 1, r);
												}
												let n = parseFloat(e);
												if (!isNaN(e - n)) return new Date(n);
												if ((t = e.match(B)))
													return (function (e) {
														let t = new Date(0),
															n = 0,
															r = 0,
															o = e[8] ? t.setUTCFullYear : t.setFullYear,
															i = e[8] ? t.setUTCHours : t.setHours;
														e[9] &&
															((n = Number(e[9] + e[10])),
															(r = Number(e[9] + e[11]))),
															o.call(
																t,
																Number(e[1]),
																Number(e[2]) - 1,
																Number(e[3]),
															);
														let s = Number(e[4] || 0) - n,
															l = Number(e[5] || 0) - r,
															a = Number(e[6] || 0),
															u = Math.floor(
																1e3 * parseFloat("0." + (e[7] || 0)),
															);
														return i.call(t, s, l, a, u), t;
													})(t);
											}
											let t = new Date(e);
											if (!eo(t))
												throw Error(`Unable to convert "${e}" into a date`);
											return t;
										})(e),
										l = (function e(t, n) {
											var r;
											let o =
												((r = t),
												(0, i.ɵfindLocaleData)(r)[i.ɵLocaleDataIndex.LocaleId]);
											if (((U[o] = U[o] || {}), U[o][n])) return U[o][n];
											let s = "";
											switch (n) {
												case "shortDate":
													s = P(t, O.Short);
													break;
												case "mediumDate":
													s = P(t, O.Medium);
													break;
												case "longDate":
													s = P(t, O.Long);
													break;
												case "fullDate":
													s = P(t, O.Full);
													break;
												case "shortTime":
													s = T(t, O.Short);
													break;
												case "mediumTime":
													s = T(t, O.Medium);
													break;
												case "longTime":
													s = T(t, O.Long);
													break;
												case "fullTime":
													s = T(t, O.Full);
													break;
												case "short":
													let l = e(t, "shortTime"),
														a = e(t, "shortDate");
													s = Z(k(t, O.Short), [l, a]);
													break;
												case "medium":
													let u = e(t, "mediumTime"),
														d = e(t, "mediumDate");
													s = Z(k(t, O.Medium), [u, d]);
													break;
												case "long":
													let c = e(t, "longTime"),
														f = e(t, "longDate");
													s = Z(k(t, O.Long), [c, f]);
													break;
												case "full":
													let h = e(t, "fullTime"),
														p = e(t, "fullDate");
													s = Z(k(t, O.Full), [h, p]);
											}
											return s && (U[o][n] = s), s;
										})(n, t);
									t = l || t;
									let a = [];
									for (; t; )
										if ((o = H.exec(t))) {
											a = a.concat(o.slice(1));
											let e = a.pop();
											if (!e) break;
											t = e;
										} else {
											a.push(t);
											break;
										}
									let u = s.getTimezoneOffset();
									r &&
										((u = er(r, u)),
										(s = (function (e, t, n) {
											var r, o;
											let i = e.getTimezoneOffset(),
												s = er(t, i);
											return (
												(r = e),
												(o = (n ? -1 : 1) * (s - i)),
												(r = new Date(r.getTime())).setMinutes(
													r.getMinutes() + o,
												),
												r
											);
										})(s, r, !0)));
									let d = "";
									return (
										a.forEach((e) => {
											let t = (function (e) {
												let t;
												if (en[e]) return en[e];
												switch (e) {
													case "G":
													case "GG":
													case "GGG":
														t = K(q.Eras, E.Abbreviated);
														break;
													case "GGGG":
														t = K(q.Eras, E.Wide);
														break;
													case "GGGGG":
														t = K(q.Eras, E.Narrow);
														break;
													case "y":
														t = Q(W.FullYear, 1, 0, !1, !0);
														break;
													case "yy":
														t = Q(W.FullYear, 2, 0, !0, !0);
														break;
													case "yyy":
														t = Q(W.FullYear, 3, 0, !1, !0);
														break;
													case "yyyy":
														t = Q(W.FullYear, 4, 0, !1, !0);
														break;
													case "Y":
														t = et(1);
														break;
													case "YY":
														t = et(2, !0);
														break;
													case "YYY":
														t = et(3);
														break;
													case "YYYY":
														t = et(4);
														break;
													case "M":
													case "L":
														t = Q(W.Month, 1, 1);
														break;
													case "MM":
													case "LL":
														t = Q(W.Month, 2, 1);
														break;
													case "MMM":
														t = K(q.Months, E.Abbreviated);
														break;
													case "MMMM":
														t = K(q.Months, E.Wide);
														break;
													case "MMMMM":
														t = K(q.Months, E.Narrow);
														break;
													case "LLL":
														t = K(q.Months, E.Abbreviated, S.Standalone);
														break;
													case "LLLL":
														t = K(q.Months, E.Wide, S.Standalone);
														break;
													case "LLLLL":
														t = K(q.Months, E.Narrow, S.Standalone);
														break;
													case "w":
														t = ee(1);
														break;
													case "ww":
														t = ee(2);
														break;
													case "W":
														t = ee(1, !0);
														break;
													case "d":
														t = Q(W.Date, 1);
														break;
													case "dd":
														t = Q(W.Date, 2);
														break;
													case "c":
													case "cc":
														t = Q(W.Day, 1);
														break;
													case "ccc":
														t = K(q.Days, E.Abbreviated, S.Standalone);
														break;
													case "cccc":
														t = K(q.Days, E.Wide, S.Standalone);
														break;
													case "ccccc":
														t = K(q.Days, E.Narrow, S.Standalone);
														break;
													case "cccccc":
														t = K(q.Days, E.Short, S.Standalone);
														break;
													case "E":
													case "EE":
													case "EEE":
														t = K(q.Days, E.Abbreviated);
														break;
													case "EEEE":
														t = K(q.Days, E.Wide);
														break;
													case "EEEEE":
														t = K(q.Days, E.Narrow);
														break;
													case "EEEEEE":
														t = K(q.Days, E.Short);
														break;
													case "a":
													case "aa":
													case "aaa":
														t = K(q.DayPeriods, E.Abbreviated);
														break;
													case "aaaa":
														t = K(q.DayPeriods, E.Wide);
														break;
													case "aaaaa":
														t = K(q.DayPeriods, E.Narrow);
														break;
													case "b":
													case "bb":
													case "bbb":
														t = K(
															q.DayPeriods,
															E.Abbreviated,
															S.Standalone,
															!0,
														);
														break;
													case "bbbb":
														t = K(q.DayPeriods, E.Wide, S.Standalone, !0);
														break;
													case "bbbbb":
														t = K(q.DayPeriods, E.Narrow, S.Standalone, !0);
														break;
													case "B":
													case "BB":
													case "BBB":
														t = K(q.DayPeriods, E.Abbreviated, S.Format, !0);
														break;
													case "BBBB":
														t = K(q.DayPeriods, E.Wide, S.Format, !0);
														break;
													case "BBBBB":
														t = K(q.DayPeriods, E.Narrow, S.Format, !0);
														break;
													case "h":
														t = Q(W.Hours, 1, -12);
														break;
													case "hh":
														t = Q(W.Hours, 2, -12);
														break;
													case "H":
														t = Q(W.Hours, 1);
														break;
													case "HH":
														t = Q(W.Hours, 2);
														break;
													case "m":
														t = Q(W.Minutes, 1);
														break;
													case "mm":
														t = Q(W.Minutes, 2);
														break;
													case "s":
														t = Q(W.Seconds, 1);
														break;
													case "ss":
														t = Q(W.Seconds, 2);
														break;
													case "S":
														t = Q(W.FractionalSeconds, 1);
														break;
													case "SS":
														t = Q(W.FractionalSeconds, 2);
														break;
													case "SSS":
														t = Q(W.FractionalSeconds, 3);
														break;
													case "Z":
													case "ZZ":
													case "ZZZ":
														t = J(z.Short);
														break;
													case "ZZZZZ":
														t = J(z.Extended);
														break;
													case "O":
													case "OO":
													case "OOO":
													case "z":
													case "zz":
													case "zzz":
														t = J(z.ShortGMT);
														break;
													case "OOOO":
													case "ZZZZ":
													case "zzzz":
														t = J(z.Long);
														break;
													default:
														return null;
												}
												return (en[e] = t), t;
											})(e);
											d += t
												? t(s, n, u)
												: "''" === e
												? "'"
												: e.replace(/(^'|'$)/g, "").replace(/''/g, "'");
										}),
										d
									);
								})(t, e, o || this.locale, c);
							} catch (t) {
								throw eC(e, t.message);
							}
						}
						constructor(e, t, n) {
							(this.locale = e),
								(this.defaultTimezone = t),
								(this.defaultOptions = n);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(
							i.ɵɵdirectiveInject(i.LOCALE_ID, 16),
							i.ɵɵdirectiveInject(eA, 24),
							i.ɵɵdirectiveInject(eI, 24),
						);
					}),
						(e.ɵpipe = i.ɵɵdefinePipe({
							name: "date",
							type: e,
							pure: !0,
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let eP = /#/g;
				(() => {
					class e {
						transform(t, n, r) {
							if (null == t) return "";
							if ("object" != typeof n || null === n) throw eC(e, n);
							let o = ed(t, Object.keys(n), this._localization, r);
							return n[o].replace(eP, t.toString());
						}
						constructor(e) {
							this._localization = e;
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(i.ɵɵdirectiveInject(eu, 16));
					}),
						(e.ɵpipe = i.ɵɵdefinePipe({
							name: "i18nPlural",
							type: e,
							pure: !0,
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t, n) {
								if (null == t) return "";
								if ("object" != typeof n || "string" != typeof t)
									throw eC(e, n);
								return n.hasOwnProperty(t)
									? n[t]
									: n.hasOwnProperty("other")
									? n.other
									: "";
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "i18nSelect",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(e) {
								return JSON.stringify(e, null, 2);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "json",
								type: e,
								pure: !1,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				function eT(e, t) {
					let n = e.key,
						r = t.key;
					if (n === r) return 0;
					if (void 0 === n) return 1;
					if (void 0 === r) return -1;
					if (null === n) return 1;
					if (null === r) return -1;
					if ("string" == typeof n && "string" == typeof r)
						return n < r ? -1 : 1;
					if ("number" == typeof n && "number" == typeof r) return n - r;
					if ("boolean" == typeof n && "boolean" == typeof r)
						return n < r ? -1 : 1;
					let o = String(n),
						i = String(r);
					return o == i ? 0 : o < i ? -1 : 1;
				}
				function ek(e) {
					return !(null == e || "" === e || e != e);
				}
				function eF(e) {
					if ("string" == typeof e && !isNaN(Number(e) - parseFloat(e)))
						return Number(e);
					if ("number" != typeof e) throw Error(`${e} is not a number`);
					return e;
				}
				(() => {
					class e {
						transform(e, t = eT) {
							if (!e || (!(e instanceof Map) && "object" != typeof e))
								return null;
							!this.differ && (this.differ = this.differs.find(e).create());
							let n = this.differ.diff(e),
								r = t !== this.compareFn;
							return (
								n &&
									((this.keyValues = []),
									n.forEachItem((e) => {
										var t;
										this.keyValues.push(
											((t = e.key), { key: t, value: e.currentValue }),
										);
									})),
								(n || r) && (this.keyValues.sort(t), (this.compareFn = t)),
								this.keyValues
							);
						}
						constructor(e) {
							(this.differs = e), (this.keyValues = []), (this.compareFn = eT);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)(i.ɵɵdirectiveInject(i.KeyValueDiffers, 16));
					}),
						(e.ɵpipe = i.ɵɵdefinePipe({
							name: "keyvalue",
							type: e,
							pure: !1,
							standalone: !0,
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t, n, r) {
								if (!ek(t)) return null;
								r = r || this._locale;
								try {
									let e = eF(t);
									return (function (e, t, n) {
										let r = R(t, M.Decimal),
											o = el(r, F(t, A.MinusSign));
										return es(e, o, t, A.Group, A.Decimal, n);
									})(e, r, n);
								} catch (t) {
									throw eC(e, t.message);
								}
							}
							constructor(e) {
								this._locale = e;
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵdirectiveInject(i.LOCALE_ID, 16));
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "number",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t, n, r) {
								if (!ek(t)) return null;
								r = r || this._locale;
								try {
									let e = eF(t);
									return (function (e, t, n) {
										let r = R(t, M.Percent),
											o = el(r, F(t, A.MinusSign)),
											i = es(e, o, t, A.Group, A.Decimal, n, !0);
										return i.replace(RegExp("%", "g"), F(t, A.PercentSign));
									})(e, r, n);
								} catch (t) {
									throw eC(e, t.message);
								}
							}
							constructor(e) {
								this._locale = e;
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵdirectiveInject(i.LOCALE_ID, 16));
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "percent",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t, n = this._defaultCurrencyCode, r = "symbol", o, s) {
								if (!ek(t)) return null;
								(s = s || this._locale),
									"boolean" == typeof r &&
										(("undefined" == typeof ngDevMode || ngDevMode) &&
											console &&
											console.warn &&
											console.warn(
												'Warning: the currency pipe has been changed in Angular v5. The symbolDisplay option (third parameter) is now a string instead of a boolean. The accepted values are "code", "symbol" or "symbol-narrow".',
											),
										(r = r ? "symbol" : "code"));
								let l = n || this._defaultCurrencyCode;
								"code" !== r &&
									(l =
										"symbol" === r || "symbol-narrow" === r
											? (function (e, t, n = "en") {
													let r =
															(function (e) {
																let t = (0, i.ɵfindLocaleData)(e);
																return t[i.ɵLocaleDataIndex.Currencies];
															})(n)[e] ||
															w[e] ||
															[],
														o = r[1];
													return "narrow" === t && "string" == typeof o
														? o
														: r[0] || e;
											  })(l, "symbol" === r ? "wide" : "narrow", s)
											: r);
								try {
									let e = eF(t);
									return (function (e, t, n, r, o) {
										let i = R(t, M.Currency),
											s = el(i, F(t, A.MinusSign));
										(s.minFrac = (function (e) {
											let t;
											let n = w[e];
											return n && (t = n[2]), "number" == typeof t ? t : 2;
										})(r)),
											(s.maxFrac = s.minFrac);
										let l = es(e, s, t, A.CurrencyGroup, A.CurrencyDecimal, o);
										return l.replace("\xa4", n).replace("\xa4", "").trim();
									})(e, s, l, n, o);
								} catch (t) {
									throw eC(e, t.message);
								}
							}
							constructor(e, t = "USD") {
								(this._locale = e), (this._defaultCurrencyCode = t);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(i.LOCALE_ID, 16),
								i.ɵɵdirectiveInject(i.DEFAULT_CURRENCY_CODE, 16),
							);
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "currency",
								type: e,
								pure: !0,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							transform(t, n, r) {
								if (null == t) return null;
								if (!this.supports(t)) throw eC(e, t);
								return t.slice(n, r);
							}
							supports(e) {
								return "string" == typeof e || Array.isArray(e);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵpipe = i.ɵɵdefinePipe({
								name: "slice",
								type: e,
								pure: !1,
								standalone: !0,
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let eR = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵmod = i.ɵɵdefineNgModule({ type: e })),
						(e.ɵinj = i.ɵɵdefineInjector({})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let eN = "browser";
				function eL(e) {
					return "server" === e;
				}
				new i.Version("16.0.0");
				let e$ = (() => {
					class e {}
					return (
						(e.ɵprov = (0, i.ɵɵdefineInjectable)({
							token: e,
							providedIn: "root",
							factory: () => new eV((0, i.ɵɵinject)(d), window),
						})),
						e
					);
				})();
				class eV {
					setOffset(e) {
						Array.isArray(e) ? (this.offset = () => e) : (this.offset = e);
					}
					getScrollPosition() {
						return this.supportsScrolling()
							? [this.window.pageXOffset, this.window.pageYOffset]
							: [0, 0];
					}
					scrollToPosition(e) {
						this.supportsScrolling() && this.window.scrollTo(e[0], e[1]);
					}
					scrollToAnchor(e) {
						if (!this.supportsScrolling()) return;
						let t = (function (e, t) {
							let n = e.getElementById(t) || e.getElementsByName(t)[0];
							if (n) return n;
							if (
								"function" == typeof e.createTreeWalker &&
								e.body &&
								"function" == typeof e.body.attachShadow
							) {
								let n = e.createTreeWalker(e.body, NodeFilter.SHOW_ELEMENT),
									r = n.currentNode;
								for (; r; ) {
									let e = r.shadowRoot;
									if (e) {
										let n =
											e.getElementById(t) || e.querySelector(`[name="${t}"]`);
										if (n) return n;
									}
									r = n.nextNode();
								}
							}
							return null;
						})(this.document, e);
						t && (this.scrollToElement(t), t.focus());
					}
					setHistoryScrollRestoration(e) {
						if (this.supportScrollRestoration()) {
							let t = this.window.history;
							t && t.scrollRestoration && (t.scrollRestoration = e);
						}
					}
					scrollToElement(e) {
						let t = e.getBoundingClientRect(),
							n = t.left + this.window.pageXOffset,
							r = t.top + this.window.pageYOffset,
							o = this.offset();
						this.window.scrollTo(n - o[0], r - o[1]);
					}
					supportScrollRestoration() {
						try {
							if (!this.supportsScrolling()) return !1;
							let e =
								eB(this.window.history) ||
								eB(Object.getPrototypeOf(this.window.history));
							return !!e && !!(e.writable || e.set);
						} catch (e) {
							return !1;
						}
					}
					supportsScrolling() {
						try {
							return (
								!!this.window &&
								!!this.window.scrollTo &&
								"pageXOffset" in this.window
							);
						} catch (e) {
							return !1;
						}
					}
					constructor(e, t) {
						(this.document = e),
							(this.window = t),
							(this.offset = () => [0, 0]);
					}
				}
				function eB(e) {
					return Object.getOwnPropertyDescriptor(e, "scrollRestoration");
				}
				class eU {}
				function eH(e, t) {
					return ez(e) ? new URL(e) : new URL(e, t.location.href);
				}
				function ez(e) {
					return /^https?:\/\//.test(e);
				}
				function eW(e) {
					return ez(e) ? new URL(e).hostname : e;
				}
				let eq = (e) => e.src,
					eG = new i.InjectionToken("ImageLoader", {
						providedIn: "root",
						factory: () => eq,
					});
				function eZ(e, t) {
					return function (n) {
						var s;
						!(function (e) {
							let t = "string" == typeof e;
							if (!t || "" === e.trim()) return !1;
							try {
								return new URL(e), !0;
							} catch (e) {
								return !1;
							}
						})(n) &&
							(function (e, t) {
								throw new i.ɵRuntimeError(
									2959,
									ngDevMode &&
										`Image loader has detected an invalid path (\`${e}\`). To fix this, supply a path using one of the following formats: ${t.join(
											" or ",
										)}`,
								);
							})(n, t || []),
							(n = (s = n).endsWith("/") ? s.slice(0, -1) : s);
						let l = [
							{
								provide: eG,
								useValue: (t) => {
									var s;
									return (
										ez(t.src) &&
											(function (e, t) {
												throw new i.ɵRuntimeError(
													2959,
													ngDevMode &&
														`Image loader has detected a \`<img>\` tag with an invalid \`ngSrc\` attribute: ${t}. This image loader expects \`ngSrc\` to be a relative URL - however the provided value is an absolute URL. To fix this, provide \`ngSrc\` as a path relative to the base URL configured for this loader (\`${e}\`).`,
												);
											})(n, t.src),
										e(
											n,
											o._(r._({}, t), {
												src: (s = t.src).startsWith("/") ? s.slice(1) : s,
											}),
										)
									);
								},
							},
						];
						return l;
					};
				}
				eZ(
					function (e, t) {
						let n = "format=auto";
						return (
							t.width && (n += `,width=${t.width}`),
							`${e}/cdn-cgi/image/${n}/${t.src}`
						);
					},
					ngDevMode
						? ["https://<ZONE>/cdn-cgi/image/<OPTIONS>/<SOURCE-IMAGE>"]
						: void 0,
				);
				let eY = /https?\:\/\/[^\/]+\.cloudinary\.com\/.+/;
				eZ(
					function (e, t) {
						let n = "f_auto,q_auto";
						return (
							t.width && (n += `,w_${t.width}`),
							`${e}/image/upload/${n}/${t.src}`
						);
					},
					ngDevMode
						? [
								"https://res.cloudinary.com/mysite",
								"https://mysite.cloudinary.com",
								"https://subdomain.mysite.com",
						  ]
						: void 0,
				);
				let eQ = /https?\:\/\/[^\/]+\.imagekit\.io\/.+/;
				eZ(
					function (e, t) {
						let n;
						let { src: r, width: o } = t;
						if (o) {
							let t = `tr:w-${o}`;
							n = [e, t, r];
						} else n = [e, r];
						return n.join("/");
					},
					ngDevMode
						? ["https://ik.imagekit.io/mysite", "https://subdomain.mysite.com"]
						: void 0,
				);
				let eK = /https?\:\/\/[^\/]+\.imgix\.net\/.+/;
				eZ(
					function (e, t) {
						let n = new URL(`${e}/${t.src}`);
						return (
							n.searchParams.set("auto", "format"),
							t.width && n.searchParams.set("w", t.width.toString()),
							n.href
						);
					},
					ngDevMode ? ["https://somepath.imgix.net/"] : void 0,
				);
				function eJ(e, t = !0) {
					let n = t
						? `(activated on an <img> element with the \`ngSrc="${e}"\`) `
						: "";
					return `The NgOptimizedImage directive ${n}has detected that`;
				}
				function eX(e) {
					if (!ngDevMode)
						throw new i.ɵRuntimeError(
							2958,
							`Unexpected invocation of the ${e} in the prod mode. Please make sure that the prod mode is enabled for production builds.`,
						);
				}
				let e0 = (() => {
					class e {
						initPerformanceObserver() {
							let e = new PerformanceObserver((e) => {
								var t, n;
								let r = e.getEntries();
								if (0 === r.length) return;
								let o = r[r.length - 1],
									s =
										null !==
											(n =
												null === (t = o.element) || void 0 === t
													? void 0
													: t.src) && void 0 !== n
											? n
											: "";
								if (s.startsWith("data:") || s.startsWith("blob:")) return;
								let l = this.images.get(s);
								l &&
									!this.alreadyWarned.has(s) &&
									(this.alreadyWarned.add(s),
									(function (e) {
										let t = eJ(e);
										console.warn(
											(0, i.ɵformatRuntimeError)(
												2955,
												`${t} this image is the Largest Contentful Paint (LCP) element but was not marked "priority". This image should be marked "priority" in order to prioritize its loading. To fix this, add the "priority" attribute.`,
											),
										);
									})(s));
							});
							return (
								e.observe({ type: "largest-contentful-paint", buffered: !0 }), e
							);
						}
						registerImage(e, t) {
							this.observer && this.images.set(eH(e, this.window).href, t);
						}
						unregisterImage(e) {
							this.observer && this.images.delete(eH(e, this.window).href);
						}
						ngOnDestroy() {
							this.observer &&
								(this.observer.disconnect(),
								this.images.clear(),
								this.alreadyWarned.clear());
						}
						constructor() {
							(this.images = new Map()),
								(this.alreadyWarned = new Set()),
								(this.window = null),
								(this.observer = null),
								eX("LCP checker");
							let e = (0, i.inject)(d).defaultView;
							void 0 !== e &&
								"undefined" != typeof PerformanceObserver &&
								((this.window = e),
								(this.observer = this.initPerformanceObserver()));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let e1 = new Set(["localhost", "127.0.0.1", "0.0.0.0"]),
					e5 = new i.InjectionToken("PRECONNECT_CHECK_BLOCKLIST"),
					e2 = (() => {
						class e {
							populateBlocklist(e) {
								Array.isArray(e)
									? (function e(t, n) {
											for (let r of t) Array.isArray(r) ? e(r, n) : n(r);
									  })(e, (e) => {
											this.blocklist.add(eW(e));
									  })
									: this.blocklist.add(eW(e));
							}
							assertPreconnect(e, t) {
								if (!this.window) return;
								let n = eH(e, this.window);
								!(
									this.blocklist.has(n.hostname) ||
									this.alreadySeen.has(n.origin)
								) &&
									(this.alreadySeen.add(n.origin),
									!this.preconnectLinks &&
										(this.preconnectLinks = this.queryPreconnectLinks()),
									!this.preconnectLinks.has(n.origin) &&
										console.warn(
											(0, i.ɵformatRuntimeError)(
												2956,
												`${eJ(t)} there is no preconnect tag present for this image. Preconnecting to the origin(s) that serve priority images ensures that these images are delivered as soon as possible. To fix this, please add the following element into the <head> of the document:
  <link rel="preconnect" href="${n.origin}">`,
											),
										));
							}
							queryPreconnectLinks() {
								let e = new Set(),
									t = Array.from(
										this.document.querySelectorAll("link[rel=preconnect]"),
									);
								for (let n of t) {
									let t = eH(n.href, this.window);
									e.add(t.origin);
								}
								return e;
							}
							ngOnDestroy() {
								var e;
								null === (e = this.preconnectLinks) ||
									void 0 === e ||
									e.clear(),
									this.alreadySeen.clear();
							}
							constructor() {
								(this.document = (0, i.inject)(d)),
									(this.preconnectLinks = null),
									(this.alreadySeen = new Set()),
									(this.window = null),
									(this.blocklist = new Set(e1)),
									eX("preconnect link checker");
								let e = this.document.defaultView;
								void 0 !== e && (this.window = e);
								let t = (0, i.inject)(e5, { optional: !0 });
								t && this.populateBlocklist(t);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let e3 = new i.InjectionToken("NG_OPTIMIZED_PRELOADED_IMAGES", {
						providedIn: "root",
						factory: () => new Set(),
					}),
					e4 = (() => {
						class e {
							createPreloadLinkTag(e, t, n, r) {
								if (ngDevMode && this.preloadedImages.size >= 5)
									throw new i.ɵRuntimeError(
										2961,
										ngDevMode &&
											'The `NgOptimizedImage` directive has detected that more than 5 images were marked as priority. This might negatively affect an overall performance of the page. To fix this, remove the "priority" attribute from images with less priority.',
									);
								if (this.preloadedImages.has(t)) return;
								this.preloadedImages.add(t);
								let o = e.createElement("link");
								e.setAttribute(o, "as", "image"),
									e.setAttribute(o, "href", t),
									e.setAttribute(o, "rel", "preload"),
									e.setAttribute(o, "fetchpriority", "high"),
									r && e.setAttribute(o, "imageSizes", r),
									n && e.setAttribute(o, "imageSrcset", n),
									e.appendChild(this.document.head, o);
							}
							constructor() {
								(this.preloadedImages = (0, i.inject)(e3)),
									(this.document = (0, i.inject)(d));
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let e6 = /^((\s*\d+w\s*(,|$)){1,})$/,
					e8 = /^((\s*\d+(\.\d+)?x\s*(,|$)){1,})$/,
					e7 = [1, 2],
					e9 = [
						{
							name: "Imgix",
							testUrl: function (e) {
								return eK.test(e);
							},
						},
						{
							name: "ImageKit",
							testUrl: function (e) {
								return eQ.test(e);
							},
						},
						{
							name: "Cloudinary",
							testUrl: function (e) {
								return eY.test(e);
							},
						},
					],
					te = {
						breakpoints: [
							16, 32, 48, 64, 96, 128, 256, 384, 640, 750, 828, 1080, 1200,
							1920, 2048, 3840,
						],
					},
					tt = new i.InjectionToken("ImageConfig", {
						providedIn: "root",
						factory: () => te,
					});
				function tn(e) {
					return "string" == typeof e ? parseInt(e, 10) : e;
				}
				function tr(e) {
					return null != e && "false" != `${e}`;
				}
				(() => {
					class e {
						set width(e) {
							ngDevMode && ti(this, e, "width"), (this._width = tn(e));
						}
						get width() {
							return this._width;
						}
						set height(e) {
							ngDevMode && ti(this, e, "height"), (this._height = tn(e));
						}
						get height() {
							return this._height;
						}
						set priority(e) {
							this._priority = tr(e);
						}
						get priority() {
							return this._priority;
						}
						set disableOptimizedSrcset(e) {
							this._disableOptimizedSrcset = tr(e);
						}
						get disableOptimizedSrcset() {
							return this._disableOptimizedSrcset;
						}
						set fill(e) {
							this._fill = tr(e);
						}
						get fill() {
							return this._fill;
						}
						ngOnInit() {
							if (ngDevMode) {
								if (
									(to(this, "ngSrc", this.ngSrc),
									(function (e, t) {
										if (null == t) return;
										to(e, "ngSrcset", t);
										let n = e6.test(t),
											r = e8.test(t);
										r &&
											(function (e, t) {
												let n = t
													.split(",")
													.every((e) => "" === e || 3 >= parseFloat(e));
												if (!n)
													throw new i.ɵRuntimeError(
														2952,
														`${eJ(
															e.ngSrc,
														)} the \`ngSrcset\` contains an unsupported image density:\`${t}\`. NgOptimizedImage generally recommends a max image density of 2x but supports image densities up to 3x. The human eye cannot distinguish between image densities greater than 2x - which makes them unnecessary for most use cases. Images that will be pinch-zoomed are typically the primary use case for 3x images. Please remove the high density descriptor and try again.`,
													);
											})(e, t);
										if (!(n || r))
											throw new i.ɵRuntimeError(
												2952,
												`${eJ(
													e.ngSrc,
												)} \`ngSrcset\` has an invalid value (\`${t}\`). To fix this, supply \`ngSrcset\` using a comma-separated list of one or more width descriptors (e.g. "100w, 200w") or density descriptors (e.g. "1x, 2x").`,
											);
									})(this, this.ngSrcset),
									(function (e) {
										if (e.src)
											throw new i.ɵRuntimeError(
												2950,
												`${eJ(
													e.ngSrc,
												)} both \`src\` and \`ngSrc\` have been set. Supplying both of these attributes breaks lazy loading. The NgOptimizedImage directive sets \`src\` itself based on the value of \`ngSrc\`. To fix this, please remove the \`src\` attribute.`,
											);
									})(this),
									this.ngSrcset &&
										(function (e) {
											if (e.srcset)
												throw new i.ɵRuntimeError(
													2951,
													`${eJ(
														e.ngSrc,
													)} both \`srcset\` and \`ngSrcset\` have been set. Supplying both of these attributes breaks lazy loading. The NgOptimizedImage directive sets \`srcset\` itself based on the value of \`ngSrcset\`. To fix this, please remove the \`srcset\` attribute.`,
												);
										})(this),
									(function (e) {
										let t = e.ngSrc.trim();
										if (t.startsWith("data:"))
											throw (
												(t.length > 50 && (t = t.substring(0, 50) + "..."),
												new i.ɵRuntimeError(
													2952,
													`${eJ(
														e.ngSrc,
														!1,
													)} \`ngSrc\` is a Base64-encoded string (${t}). NgOptimizedImage does not support Base64-encoded strings. To fix this, disable the NgOptimizedImage directive for this element by removing \`ngSrc\` and using a standard \`src\` attribute instead.`,
												))
											);
									})(this),
									(function (e) {
										let t = e.ngSrc.trim();
										if (t.startsWith("blob:"))
											throw new i.ɵRuntimeError(
												2952,
												`${eJ(
													e.ngSrc,
												)} \`ngSrc\` was set to a blob URL (${t}). Blob URLs are not supported by the NgOptimizedImage directive. To fix this, disable the NgOptimizedImage directive for this element by removing \`ngSrc\` and using a regular \`src\` attribute instead.`,
											);
									})(this),
									this.fill
										? ((function (e) {
												if (e.width || e.height)
													throw new i.ɵRuntimeError(
														2952,
														`${eJ(
															e.ngSrc,
														)} the attributes \`height\` and/or \`width\` are present along with the \`fill\` attribute. Because \`fill\` mode causes an image to fill its containing element, the size attributes have no effect and should be removed.`,
													);
										  })(this),
										  (function (e, t, n) {
												let r = n.listen(t, "load", () => {
													r();
													let n = t.clientHeight;
													e.fill &&
														0 === n &&
														console.warn(
															(0, i.ɵformatRuntimeError)(
																2952,
																`${eJ(
																	e.ngSrc,
																)} the height of the fill-mode image is zero. This is likely because the containing element does not have the CSS 'position' property set to one of the following: "relative", "fixed", or "absolute". To fix this problem, make sure the container element has the CSS 'position' property defined and the height of the element is not zero.`,
															),
														);
												});
										  })(this, this.imgElement, this.renderer))
										: ((function (e) {
												let t = [];
												if (
													(void 0 === e.width && t.push("width"),
													void 0 === e.height && t.push("height"),
													t.length > 0)
												)
													throw new i.ɵRuntimeError(
														2954,
														`${eJ(
															e.ngSrc,
														)} these required attributes are missing: ${t
															.map((e) => `"${e}"`)
															.join(
																", ",
															)}. Including "width" and "height" attributes will prevent image-related layout shifts. To fix this, include "width" and "height" attributes on the image tag or turn on "fill" mode with the \`fill\` attribute.`,
													);
										  })(this),
										  (function (e, t, n) {
												let r = n.listen(t, "load", () => {
													r();
													let n = window.getComputedStyle(t),
														o = parseFloat(n.getPropertyValue("width")),
														s = parseFloat(n.getPropertyValue("height")),
														l = n.getPropertyValue("box-sizing");
													if ("border-box" === l) {
														let e = n.getPropertyValue("padding-top"),
															t = n.getPropertyValue("padding-right"),
															r = n.getPropertyValue("padding-bottom"),
															i = n.getPropertyValue("padding-left");
														(o -= parseFloat(t) + parseFloat(i)),
															(s -= parseFloat(e) + parseFloat(r));
													}
													let a = o / s,
														u = 0 !== o && 0 !== s,
														d = t.naturalWidth,
														c = t.naturalHeight,
														f = d / c,
														h = e.width,
														p = e.height,
														m = h / p,
														g = Math.abs(m - f) > 0.1,
														v = u && Math.abs(f - a) > 0.1;
													if (g)
														console.warn(
															(0, i.ɵformatRuntimeError)(
																2952,
																`${eJ(e.ngSrc)} the aspect ratio of the image does not match the aspect ratio indicated by the width and height attributes. 
Intrinsic image size: ${d}w x ${c}h (aspect-ratio: ${f}). 
Supplied width and height attributes: ${h}w x ${p}h (aspect-ratio: ${m}). 
To fix this, update the width and height attributes.`,
															),
														);
													else if (v)
														console.warn(
															(0, i.ɵformatRuntimeError)(
																2952,
																`${eJ(e.ngSrc)} the aspect ratio of the rendered image does not match the image's intrinsic aspect ratio. 
Intrinsic image size: ${d}w x ${c}h (aspect-ratio: ${f}). 
Rendered image size: ${o}w x ${s}h (aspect-ratio: ${a}). 
This issue can occur if "width" and "height" attributes are added to an image without updating the corresponding image styling. To fix this, adjust image styling. In most cases, adding "height: auto" or "width: auto" to the image styling will fix this issue.`,
															),
														);
													else if (!e.ngSrcset && u) {
														let t = 2 * o,
															n = 2 * s,
															r = d - t >= 1e3,
															l = c - n >= 1e3;
														(r || l) &&
															console.warn(
																(0, i.ɵformatRuntimeError)(
																	2960,
																	`${eJ(e.ngSrc)} the intrinsic image is significantly larger than necessary. 
Rendered image size: ${o}w x ${s}h. 
Intrinsic image size: ${d}w x ${c}h. 
Recommended intrinsic image size: ${t}w x ${n}h. 
Note: Recommended intrinsic image size is calculated assuming a maximum DPR of 2. To improve loading time, resize the image or consider using the "ngSrcset" and "sizes" attributes.`,
																),
															);
													}
												});
										  })(this, this.imgElement, this.renderer)),
									(function (e) {
										if (e.loading && e.priority)
											throw new i.ɵRuntimeError(
												2952,
												`${eJ(
													e.ngSrc,
												)} the \`loading\` attribute was used on an image that was marked "priority". Setting \`loading\` on priority images is not allowed because these images will always be eagerly loaded. To fix this, remove the “loading” attribute from the priority image.`,
											);
										if (
											"string" == typeof e.loading &&
											!["auto", "eager", "lazy"].includes(e.loading)
										)
											throw new i.ɵRuntimeError(
												2952,
												`${eJ(
													e.ngSrc,
												)} the \`loading\` attribute has an invalid value (\`${
													e.loading
												}\`). To fix this, provide a valid value ("lazy", "eager", or "auto").`,
											);
									})(this),
									!this.ngSrcset &&
										(function (e) {
											let t = e.sizes;
											if (null == t ? void 0 : t.match(/((\)|,)\s|^)\d+px/))
												throw new i.ɵRuntimeError(
													2952,
													`${eJ(
														e.ngSrc,
														!1,
													)} \`sizes\` was set to a string including pixel values. For automatic \`srcset\` generation, \`sizes\` must only include responsive values, such as \`sizes="50vw"\` or \`sizes="(min-width: 768px) 50vw, 100vw"\`. To fix this, modify the \`sizes\` attribute, or provide your own \`ngSrcset\` value directly.`,
												);
										})(this),
									(function (e, t) {
										if (t === eq) {
											let t = "";
											for (let n of e9)
												if (n.testUrl(e)) {
													t = n.name;
													break;
												}
											t &&
												console.warn(
													(0, i.ɵformatRuntimeError)(
														2962,
														`NgOptimizedImage: It looks like your images may be hosted on the ${t} CDN, but your app is not using Angular's built-in loader for that CDN. We recommend switching to use the built-in by calling \`provide${t}Loader()\` in your \`providers\` and passing it your instance's base URL. If you don't want to use the built-in loader, define a custom loader function using IMAGE_LOADER to silence this warning.`,
													),
												);
										}
									})(this.ngSrc, this.imageLoader),
									(function (e, t) {
										e.ngSrcset &&
											t === eq &&
											console.warn(
												(0, i.ɵformatRuntimeError)(
													2963,
													`${eJ(
														e.ngSrc,
													)} the \`ngSrcset\` attribute is present but no image loader is configured (i.e. the default one is being used), which would result in the same image being used for all configured sizes. To fix this, provide a loader or remove the \`ngSrcset\` attribute from the image.`,
												),
											);
									})(this, this.imageLoader),
									(function (e, t) {
										e.loaderParams &&
											t === eq &&
											console.warn(
												(0, i.ɵformatRuntimeError)(
													2963,
													`${eJ(
														e.ngSrc,
													)} the \`loaderParams\` attribute is present but no image loader is configured (i.e. the default one is being used), which means that the loaderParams data will not be consumed and will not affect the URL. To fix this, provide a custom loader or remove the \`loaderParams\` attribute from the image.`,
												),
											);
									})(this, this.imageLoader),
									this.priority)
								) {
									let e = this.injector.get(e2);
									e.assertPreconnect(this.getRewrittenSrc(), this.ngSrc);
								} else if (null !== this.lcpObserver) {
									let e = this.injector.get(i.NgZone);
									e.runOutsideAngular(() => {
										this.lcpObserver.registerImage(
											this.getRewrittenSrc(),
											this.ngSrc,
										);
									});
								}
							}
							this.setHostAttributes();
						}
						setHostAttributes() {
							let e;
							this.fill
								? !this.sizes && (this.sizes = "100vw")
								: (this.setHostAttribute("width", this.width.toString()),
								  this.setHostAttribute("height", this.height.toString())),
								this.setHostAttribute("loading", this.getLoadingBehavior()),
								this.setHostAttribute("fetchpriority", this.getFetchPriority()),
								this.setHostAttribute("ng-img", "true");
							let t = this.getRewrittenSrc();
							this.setHostAttribute("src", t);
							this.sizes && this.setHostAttribute("sizes", this.sizes),
								this.ngSrcset
									? (e = this.getRewrittenSrcset())
									: this.shouldGenerateAutomaticSrcset() &&
									  (e = this.getAutomaticSrcset()),
								e && this.setHostAttribute("srcset", e),
								this.isServer &&
									this.priority &&
									this.preloadLinkChecker.createPreloadLinkTag(
										this.renderer,
										t,
										e,
										this.sizes,
									);
						}
						ngOnChanges(e) {
							ngDevMode &&
								(function (e, t, n) {
									n.forEach((n) => {
										let r = t.hasOwnProperty(n);
										if (r && !t[n].isFirstChange()) {
											var o, s;
											let r;
											"ngSrc" === n && (e = { ngSrc: t[n].previousValue });
											throw (
												((o = e),
												(r =
													"width" === (s = n) || "height" === s
														? `Changing \`${s}\` may result in different attribute value applied to the underlying image element and cause layout shifts on a page.`
														: `Changing the \`${s}\` would have no effect on the underlying image element, because the resource loading has already occurred.`),
												new i.ɵRuntimeError(
													2953,
													`${eJ(
														o.ngSrc,
													)} \`${s}\` was updated after initialization. The NgOptimizedImage directive will not react to this input change. ${r} To fix this, either switch \`${s}\` to a static value or wrap the image element in an *ngIf that is gated on the necessary value.`,
												))
											);
										}
									});
								})(this, e, [
									"ngSrc",
									"ngSrcset",
									"width",
									"height",
									"priority",
									"fill",
									"loading",
									"sizes",
									"loaderParams",
									"disableOptimizedSrcset",
								]);
						}
						callImageLoader(e) {
							return (
								this.loaderParams && (e.loaderParams = this.loaderParams),
								this.imageLoader(e)
							);
						}
						getLoadingBehavior() {
							return this.priority || void 0 === this.loading
								? this.priority
									? "eager"
									: "lazy"
								: this.loading;
						}
						getFetchPriority() {
							return this.priority ? "high" : "auto";
						}
						getRewrittenSrc() {
							if (!this._renderedSrc) {
								let e = { src: this.ngSrc };
								this._renderedSrc = this.callImageLoader(e);
							}
							return this._renderedSrc;
						}
						getRewrittenSrcset() {
							let e = e6.test(this.ngSrcset),
								t = this.ngSrcset
									.split(",")
									.filter((e) => "" !== e)
									.map((t) => {
										t = t.trim();
										let n = e ? parseFloat(t) : parseFloat(t) * this.width;
										return `${this.callImageLoader({
											src: this.ngSrc,
											width: n,
										})} ${t}`;
									});
							return t.join(", ");
						}
						getAutomaticSrcset() {
							return this.sizes
								? this.getResponsiveSrcset()
								: this.getFixedSrcset();
						}
						getResponsiveSrcset() {
							var e;
							let { breakpoints: t } = this.config,
								n = t;
							(null === (e = this.sizes) || void 0 === e
								? void 0
								: e.trim()) === "100vw" && (n = t.filter((e) => e >= 640));
							let r = n.map(
								(e) =>
									`${this.callImageLoader({
										src: this.ngSrc,
										width: e,
									})} ${e}w`,
							);
							return r.join(", ");
						}
						getFixedSrcset() {
							let e = e7.map(
								(e) =>
									`${this.callImageLoader({
										src: this.ngSrc,
										width: this.width * e,
									})} ${e}x`,
							);
							return e.join(", ");
						}
						shouldGenerateAutomaticSrcset() {
							return (
								!this._disableOptimizedSrcset &&
								!this.srcset &&
								this.imageLoader !== eq &&
								!(this.width > 1920 || this.height > 1080)
							);
						}
						ngOnDestroy() {
							ngDevMode &&
								!this.priority &&
								null !== this._renderedSrc &&
								null !== this.lcpObserver &&
								this.lcpObserver.unregisterImage(this._renderedSrc);
						}
						setHostAttribute(e, t) {
							this.renderer.setAttribute(this.imgElement, e, t);
						}
						constructor() {
							(this.imageLoader = (0, i.inject)(eG)),
								(this.config = (function (e) {
									let t = {};
									return (
										e.breakpoints &&
											(t.breakpoints = e.breakpoints.sort((e, t) => e - t)),
										Object.assign({}, te, e, t)
									);
								})((0, i.inject)(tt))),
								(this.renderer = (0, i.inject)(i.Renderer2)),
								(this.imgElement = (0, i.inject)(i.ElementRef).nativeElement),
								(this.injector = (0, i.inject)(i.Injector)),
								(this.isServer = eL((0, i.inject)(i.PLATFORM_ID))),
								(this.preloadLinkChecker = (0, i.inject)(e4)),
								(this.lcpObserver = ngDevMode ? this.injector.get(e0) : null),
								(this._renderedSrc = null),
								(this._priority = !1),
								(this._disableOptimizedSrcset = !1),
								(this._fill = !1);
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)();
					}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["img", "ngSrc", ""]],
							hostVars: 8,
							hostBindings: function (e, t) {
								2 & e &&
									i.ɵɵstyleProp("position", t.fill ? "absolute" : null)(
										"width",
										t.fill ? "100%" : null,
									)("height", t.fill ? "100%" : null)(
										"inset",
										t.fill ? "0px" : null,
									);
							},
							inputs: {
								ngSrc: "ngSrc",
								ngSrcset: "ngSrcset",
								sizes: "sizes",
								width: "width",
								height: "height",
								loading: "loading",
								priority: "priority",
								loaderParams: "loaderParams",
								disableOptimizedSrcset: "disableOptimizedSrcset",
								fill: "fill",
								src: "src",
								srcset: "srcset",
							},
							standalone: !0,
							features: [i.ɵɵNgOnChangesFeature],
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				function to(e, t, n) {
					let r = "string" == typeof n,
						o = r && "" === n.trim();
					if (!r || o)
						throw new i.ɵRuntimeError(
							2952,
							`${eJ(
								e.ngSrc,
							)} \`${t}\` has an invalid value (\`${n}\`). To fix this, change the value to a non-empty string.`,
						);
				}
				function ti(e, t, n) {
					let r = "number" == typeof t && t > 0,
						o =
							"string" == typeof t && /^\d+$/.test(t.trim()) && parseInt(t) > 0;
					if (!r && !o)
						throw new i.ɵRuntimeError(
							2952,
							`${eJ(
								e.ngSrc,
							)} \`${n}\` has an invalid value (\`${t}\`). To fix this, provide \`${n}\` as a number greater than 0.`,
						);
				}
			},
			"../../node_modules/@angular/common/fesm2022/http.mjs": function (
				e,
				t,
				n,
			) {
				"use strict";
				let r;
				Object.defineProperty(t, "__esModule", { value: !0 });
				var o = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs")),
					i = n("../../node_modules/rxjs/dist/esm5/index.js"),
					s = n("../../node_modules/rxjs/dist/esm5/operators/index.js"),
					l = n.ir(n("../../node_modules/@angular/common/fesm2022/common.mjs"));
				class a {}
				class u {}
				class d {
					has(e) {
						return this.init(), this.headers.has(e.toLowerCase());
					}
					get(e) {
						this.init();
						let t = this.headers.get(e.toLowerCase());
						return t && t.length > 0 ? t[0] : null;
					}
					keys() {
						return this.init(), Array.from(this.normalizedNames.values());
					}
					getAll(e) {
						return this.init(), this.headers.get(e.toLowerCase()) || null;
					}
					append(e, t) {
						return this.clone({ name: e, value: t, op: "a" });
					}
					set(e, t) {
						return this.clone({ name: e, value: t, op: "s" });
					}
					delete(e, t) {
						return this.clone({ name: e, value: t, op: "d" });
					}
					maybeSetNormalizedName(e, t) {
						!this.normalizedNames.has(t) && this.normalizedNames.set(t, e);
					}
					init() {
						this.lazyInit &&
							(this.lazyInit instanceof d
								? this.copyFrom(this.lazyInit)
								: this.lazyInit(),
							(this.lazyInit = null),
							this.lazyUpdate &&
								(this.lazyUpdate.forEach((e) => this.applyUpdate(e)),
								(this.lazyUpdate = null)));
					}
					copyFrom(e) {
						e.init(),
							Array.from(e.headers.keys()).forEach((t) => {
								this.headers.set(t, e.headers.get(t)),
									this.normalizedNames.set(t, e.normalizedNames.get(t));
							});
					}
					clone(e) {
						let t = new d();
						return (
							(t.lazyInit =
								this.lazyInit && this.lazyInit instanceof d
									? this.lazyInit
									: this),
							(t.lazyUpdate = (this.lazyUpdate || []).concat([e])),
							t
						);
					}
					applyUpdate(e) {
						let t = e.name.toLowerCase();
						switch (e.op) {
							case "a":
							case "s":
								let n = e.value;
								if (("string" == typeof n && (n = [n]), 0 === n.length)) return;
								this.maybeSetNormalizedName(e.name, t);
								let r = ("a" === e.op ? this.headers.get(t) : void 0) || [];
								r.push(...n), this.headers.set(t, r);
								break;
							case "d":
								let o = e.value;
								if (o) {
									let e = this.headers.get(t);
									if (!e) return;
									0 === (e = e.filter((e) => -1 === o.indexOf(e))).length
										? (this.headers.delete(t), this.normalizedNames.delete(t))
										: this.headers.set(t, e);
								} else this.headers.delete(t), this.normalizedNames.delete(t);
						}
					}
					forEach(e) {
						this.init(),
							Array.from(this.normalizedNames.keys()).forEach((t) =>
								e(this.normalizedNames.get(t), this.headers.get(t)),
							);
					}
					constructor(e) {
						(this.normalizedNames = new Map()),
							(this.lazyUpdate = null),
							e
								? "string" == typeof e
									? (this.lazyInit = () => {
											(this.headers = new Map()),
												e.split("\n").forEach((e) => {
													let t = e.indexOf(":");
													if (t > 0) {
														let n = e.slice(0, t),
															r = n.toLowerCase(),
															o = e.slice(t + 1).trim();
														this.maybeSetNormalizedName(n, r),
															this.headers.has(r)
																? this.headers.get(r).push(o)
																: this.headers.set(r, [o]);
													}
												});
									  })
									: (this.lazyInit = () => {
											("undefined" == typeof ngDevMode || ngDevMode) &&
												(function (e) {
													for (let [t, n] of Object.entries(e))
														if (
															!("string" == typeof n || "number" == typeof n) &&
															!Array.isArray(n)
														)
															throw Error(
																`Unexpected value of the \`${t}\` header provided. Expecting either a string, a number or an array, but got: \`${n}\`.`,
															);
												})(e),
												(this.headers = new Map()),
												Object.entries(e).forEach(([e, t]) => {
													let n;
													if (
														(n =
															"string" == typeof t
																? [t]
																: "number" == typeof t
																? [t.toString()]
																: t.map((e) => e.toString())).length > 0
													) {
														let t = e.toLowerCase();
														this.headers.set(t, n),
															this.maybeSetNormalizedName(e, t);
													}
												});
									  })
								: (this.headers = new Map());
					}
				}
				class c {
					encodeKey(e) {
						return p(e);
					}
					encodeValue(e) {
						return p(e);
					}
					decodeKey(e) {
						return decodeURIComponent(e);
					}
					decodeValue(e) {
						return decodeURIComponent(e);
					}
				}
				let f = /%(\d[a-f0-9])/gi,
					h = {
						40: "@",
						"3A": ":",
						24: "$",
						"2C": ",",
						"3B": ";",
						"3D": "=",
						"3F": "?",
						"2F": "/",
					};
				function p(e) {
					var t;
					return encodeURIComponent(e).replace(f, (e, n) =>
						null !== (t = h[n]) && void 0 !== t ? t : e,
					);
				}
				function m(e) {
					return `${e}`;
				}
				class g {
					has(e) {
						return this.init(), this.map.has(e);
					}
					get(e) {
						this.init();
						let t = this.map.get(e);
						return t ? t[0] : null;
					}
					getAll(e) {
						return this.init(), this.map.get(e) || null;
					}
					keys() {
						return this.init(), Array.from(this.map.keys());
					}
					append(e, t) {
						return this.clone({ param: e, value: t, op: "a" });
					}
					appendAll(e) {
						let t = [];
						return (
							Object.keys(e).forEach((n) => {
								let r = e[n];
								Array.isArray(r)
									? r.forEach((e) => {
											t.push({ param: n, value: e, op: "a" });
									  })
									: t.push({ param: n, value: r, op: "a" });
							}),
							this.clone(t)
						);
					}
					set(e, t) {
						return this.clone({ param: e, value: t, op: "s" });
					}
					delete(e, t) {
						return this.clone({ param: e, value: t, op: "d" });
					}
					toString() {
						return (
							this.init(),
							this.keys()
								.map((e) => {
									let t = this.encoder.encodeKey(e);
									return this.map
										.get(e)
										.map((e) => t + "=" + this.encoder.encodeValue(e))
										.join("&");
								})
								.filter((e) => "" !== e)
								.join("&")
						);
					}
					clone(e) {
						let t = new g({ encoder: this.encoder });
						return (
							(t.cloneFrom = this.cloneFrom || this),
							(t.updates = (this.updates || []).concat(e)),
							t
						);
					}
					init() {
						null === this.map && (this.map = new Map()),
							null !== this.cloneFrom &&
								(this.cloneFrom.init(),
								this.cloneFrom
									.keys()
									.forEach((e) => this.map.set(e, this.cloneFrom.map.get(e))),
								this.updates.forEach((e) => {
									switch (e.op) {
										case "a":
										case "s":
											let t =
												("a" === e.op ? this.map.get(e.param) : void 0) || [];
											t.push(m(e.value)), this.map.set(e.param, t);
											break;
										case "d":
											if (void 0 !== e.value) {
												let t = this.map.get(e.param) || [],
													n = t.indexOf(m(e.value));
												-1 !== n && t.splice(n, 1),
													t.length > 0
														? this.map.set(e.param, t)
														: this.map.delete(e.param);
											} else this.map.delete(e.param);
									}
								}),
								(this.cloneFrom = this.updates = null));
					}
					constructor(e = {}) {
						if (
							((this.updates = null),
							(this.cloneFrom = null),
							(this.encoder = e.encoder || new c()),
							e.fromString)
						) {
							if (e.fromObject)
								throw Error("Cannot specify both fromString and fromObject.");
							this.map = (function (e, t) {
								let n = new Map();
								if (e.length > 0) {
									let r = e.replace(/^\?/, "").split("&");
									r.forEach((e) => {
										let r = e.indexOf("="),
											[o, i] =
												-1 == r
													? [t.decodeKey(e), ""]
													: [
															t.decodeKey(e.slice(0, r)),
															t.decodeValue(e.slice(r + 1)),
													  ],
											s = n.get(o) || [];
										s.push(i), n.set(o, s);
									});
								}
								return n;
							})(e.fromString, this.encoder);
						} else
							e.fromObject
								? ((this.map = new Map()),
								  Object.keys(e.fromObject).forEach((t) => {
										let n = e.fromObject[t],
											r = Array.isArray(n) ? n.map(m) : [m(n)];
										this.map.set(t, r);
								  }))
								: (this.map = null);
					}
				}
				class v {
					set(e, t) {
						return this.map.set(e, t), this;
					}
					get(e) {
						return (
							!this.map.has(e) && this.map.set(e, e.defaultValue()),
							this.map.get(e)
						);
					}
					delete(e) {
						return this.map.delete(e), this;
					}
					has(e) {
						return this.map.has(e);
					}
					keys() {
						return this.map.keys();
					}
					constructor() {
						this.map = new Map();
					}
				}
				function y(e) {
					return "undefined" != typeof ArrayBuffer && e instanceof ArrayBuffer;
				}
				function b(e) {
					return "undefined" != typeof Blob && e instanceof Blob;
				}
				function _(e) {
					return "undefined" != typeof FormData && e instanceof FormData;
				}
				class j {
					serializeBody() {
						var e;
						if (null === this.body) return null;
						return y(this.body) ||
							b(this.body) ||
							_(this.body) ||
							((e = this.body),
							"undefined" != typeof URLSearchParams &&
								e instanceof URLSearchParams) ||
							"string" == typeof this.body
							? this.body
							: this.body instanceof g
							? this.body.toString()
							: "object" == typeof this.body ||
							  "boolean" == typeof this.body ||
							  Array.isArray(this.body)
							? JSON.stringify(this.body)
							: this.body.toString();
					}
					detectContentTypeHeader() {
						return null === this.body || _(this.body)
							? null
							: b(this.body)
							? this.body.type || null
							: y(this.body)
							? null
							: "string" == typeof this.body
							? "text/plain"
							: this.body instanceof g
							? "application/x-www-form-urlencoded;charset=UTF-8"
							: "object" == typeof this.body ||
							  "number" == typeof this.body ||
							  "boolean" == typeof this.body
							? "application/json"
							: null;
					}
					clone(e = {}) {
						var t;
						let n = e.method || this.method,
							r = e.url || this.url,
							o = e.responseType || this.responseType,
							i = void 0 !== e.body ? e.body : this.body,
							s =
								void 0 !== e.withCredentials
									? e.withCredentials
									: this.withCredentials,
							l =
								void 0 !== e.reportProgress
									? e.reportProgress
									: this.reportProgress,
							a = e.headers || this.headers,
							u = e.params || this.params,
							d = null !== (t = e.context) && void 0 !== t ? t : this.context;
						return (
							void 0 !== e.setHeaders &&
								(a = Object.keys(e.setHeaders).reduce(
									(t, n) => t.set(n, e.setHeaders[n]),
									a,
								)),
							e.setParams &&
								(u = Object.keys(e.setParams).reduce(
									(t, n) => t.set(n, e.setParams[n]),
									u,
								)),
							new j(n, r, i, {
								params: u,
								headers: a,
								context: d,
								reportProgress: l,
								responseType: o,
								withCredentials: s,
							})
						);
					}
					constructor(e, t, n, r) {
						let o;
						if (
							((this.url = t),
							(this.body = null),
							(this.reportProgress = !1),
							(this.withCredentials = !1),
							(this.responseType = "json"),
							(this.method = e.toUpperCase()),
							(function (e) {
								switch (e) {
									case "DELETE":
									case "GET":
									case "HEAD":
									case "OPTIONS":
									case "JSONP":
										return !1;
									default:
										return !0;
								}
							})(this.method) || r
								? ((this.body = void 0 !== n ? n : null), (o = r))
								: (o = n),
							o &&
								((this.reportProgress = !!o.reportProgress),
								(this.withCredentials = !!o.withCredentials),
								o.responseType && (this.responseType = o.responseType),
								o.headers && (this.headers = o.headers),
								o.context && (this.context = o.context),
								o.params && (this.params = o.params)),
							!this.headers && (this.headers = new d()),
							!this.context && (this.context = new v()),
							this.params)
						) {
							let e = this.params.toString();
							if (0 === e.length) this.urlWithParams = t;
							else {
								let n = t.indexOf("?"),
									r = -1 === n ? "?" : n < t.length - 1 ? "&" : "";
								this.urlWithParams = t + r + e;
							}
						} else (this.params = new g()), (this.urlWithParams = t);
					}
				}
				var x =
					(((x = x || {})[(x.Sent = 0)] = "Sent"),
					(x[(x.UploadProgress = 1)] = "UploadProgress"),
					(x[(x.ResponseHeader = 2)] = "ResponseHeader"),
					(x[(x.DownloadProgress = 3)] = "DownloadProgress"),
					(x[(x.Response = 4)] = "Response"),
					(x[(x.User = 5)] = "User"),
					x);
				class D {
					constructor(e, t = 200, n = "OK") {
						(this.headers = e.headers || new d()),
							(this.status = void 0 !== e.status ? e.status : t),
							(this.statusText = e.statusText || n),
							(this.url = e.url || null),
							(this.ok = this.status >= 200 && this.status < 300);
					}
				}
				class w extends D {
					clone(e = {}) {
						return new w({
							headers: e.headers || this.headers,
							status: void 0 !== e.status ? e.status : this.status,
							statusText: e.statusText || this.statusText,
							url: e.url || this.url || void 0,
						});
					}
					constructor(e = {}) {
						super(e), (this.type = x.ResponseHeader);
					}
				}
				class M extends D {
					clone(e = {}) {
						return new M({
							body: void 0 !== e.body ? e.body : this.body,
							headers: e.headers || this.headers,
							status: void 0 !== e.status ? e.status : this.status,
							statusText: e.statusText || this.statusText,
							url: e.url || this.url || void 0,
						});
					}
					constructor(e = {}) {
						super(e),
							(this.type = x.Response),
							(this.body = void 0 !== e.body ? e.body : null);
					}
				}
				class C extends D {
					constructor(e) {
						super(e, 0, "Unknown Error"),
							(this.name = "HttpErrorResponse"),
							(this.ok = !1),
							this.status >= 200 && this.status < 300
								? (this.message = `Http failure during parsing for ${
										e.url || "(unknown url)"
								  }`)
								: (this.message = `Http failure response for ${
										e.url || "(unknown url)"
								  }: ${e.status} ${e.statusText}`),
							(this.error = e.error || null);
					}
				}
				function S(e, t) {
					return {
						body: t,
						headers: e.headers,
						context: e.context,
						observe: e.observe,
						params: e.params,
						reportProgress: e.reportProgress,
						responseType: e.responseType,
						withCredentials: e.withCredentials,
					};
				}
				let E = (() => {
					class e {
						request(e, t, n = {}) {
							let r;
							if (e instanceof j) r = e;
							else {
								let o, i;
								o = n.headers instanceof d ? n.headers : new d(n.headers);
								n.params &&
									(i =
										n.params instanceof g
											? n.params
											: new g({ fromObject: n.params })),
									(r = new j(e, t, void 0 !== n.body ? n.body : null, {
										headers: o,
										context: n.context,
										params: i,
										reportProgress: n.reportProgress,
										responseType: n.responseType || "json",
										withCredentials: n.withCredentials,
									}));
							}
							let o = (0, i.of)(r).pipe(
								(0, s.concatMap)((e) => this.handler.handle(e)),
							);
							if (e instanceof j || "events" === n.observe) return o;
							let l = o.pipe((0, s.filter)((e) => e instanceof M));
							switch (n.observe || "body") {
								case "body":
									switch (r.responseType) {
										case "arraybuffer":
											return l.pipe(
												(0, s.map)((e) => {
													if (
														null !== e.body &&
														!(e.body instanceof ArrayBuffer)
													)
														throw Error("Response is not an ArrayBuffer.");
													return e.body;
												}),
											);
										case "blob":
											return l.pipe(
												(0, s.map)((e) => {
													if (null !== e.body && !(e.body instanceof Blob))
														throw Error("Response is not a Blob.");
													return e.body;
												}),
											);
										case "text":
											return l.pipe(
												(0, s.map)((e) => {
													if (null !== e.body && "string" != typeof e.body)
														throw Error("Response is not a string.");
													return e.body;
												}),
											);
										default:
											return l.pipe((0, s.map)((e) => e.body));
									}
								case "response":
									return l;
								default:
									throw Error(
										`Unreachable: unhandled observe type ${n.observe}}`,
									);
							}
						}
						delete(e, t = {}) {
							return this.request("DELETE", e, t);
						}
						get(e, t = {}) {
							return this.request("GET", e, t);
						}
						head(e, t = {}) {
							return this.request("HEAD", e, t);
						}
						jsonp(e, t) {
							return this.request("JSONP", e, {
								params: new g().append(t, "JSONP_CALLBACK"),
								observe: "body",
								responseType: "json",
							});
						}
						options(e, t = {}) {
							return this.request("OPTIONS", e, t);
						}
						patch(e, t, n = {}) {
							return this.request("PATCH", e, S(n, t));
						}
						post(e, t, n = {}) {
							return this.request("POST", e, S(n, t));
						}
						put(e, t, n = {}) {
							return this.request("PUT", e, S(n, t));
						}
						constructor(e) {
							this.handler = e;
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(o.ɵɵinject(a));
						}),
						(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				function O(e, t) {
					return t(e);
				}
				function A(e, t) {
					return (n, r) => t.intercept(n, { handle: (t) => e(t, r) });
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let I = new o.InjectionToken(ngDevMode ? "HTTP_INTERCEPTORS" : ""),
					P = new o.InjectionToken(ngDevMode ? "HTTP_INTERCEPTOR_FNS" : ""),
					T = new o.InjectionToken(
						ngDevMode ? "HTTP_ROOT_INTERCEPTOR_FNS" : "",
					);
				function k() {
					let e = null;
					return (t, n) => {
						if (null === e) {
							var r;
							let t =
								null !== (r = (0, o.inject)(I, { optional: !0 })) &&
								void 0 !== r
									? r
									: [];
							e = t.reduceRight(A, O);
						}
						return e(t, n);
					};
				}
				let F = (() => {
					class e extends a {
						handle(e) {
							if (null === this.chain) {
								let e = Array.from(
									new Set([
										...this.injector.get(P),
										...this.injector.get(T, []),
									]),
								);
								this.chain = e.reduceRight((e, t) => {
									var n, r, o;
									return (
										(n = e),
										(r = t),
										(o = this.injector),
										(e, t) => o.runInContext(() => r(e, (e) => n(e, t)))
									);
								}, O);
							}
							return this.chain(e, (e) => this.backend.handle(e));
						}
						constructor(e, t) {
							super(),
								(this.backend = e),
								(this.injector = t),
								(this.chain = null);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								o.ɵɵinject(u),
								o.ɵɵinject(o.EnvironmentInjector),
							);
						}),
						(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let R = 0;
				class N {}
				function L() {
					return "object" == typeof window ? window : {};
				}
				let $ = (() => {
					class e {
						nextCallback() {
							return `ng_jsonp_callback_${R++}`;
						}
						handle(e) {
							if ("JSONP" !== e.method)
								throw Error("JSONP requests must use JSONP request method.");
							if ("json" !== e.responseType)
								throw Error("JSONP requests must use Json response type.");
							if (e.headers.keys().length > 0)
								throw Error("JSONP requests do not support headers.");
							return new i.Observable((t) => {
								let n = this.nextCallback(),
									r = e.urlWithParams.replace(
										/=JSONP_CALLBACK(&|$)/,
										`=${n}$1`,
									),
									o = this.document.createElement("script");
								o.src = r;
								let i = null,
									s = !1;
								this.callbackMap[n] = (e) => {
									delete this.callbackMap[n], (i = e), (s = !0);
								};
								let l = () => {
									o.parentNode && o.parentNode.removeChild(o),
										delete this.callbackMap[n];
								};
								return (
									o.addEventListener("load", (e) => {
										this.resolvedPromise.then(() => {
											if ((l(), !s)) {
												t.error(
													new C({
														url: r,
														status: 0,
														statusText: "JSONP Error",
														error: Error(
															"JSONP injected script did not invoke callback.",
														),
													}),
												);
												return;
											}
											t.next(
												new M({
													body: i,
													status: 200,
													statusText: "OK",
													url: r,
												}),
											),
												t.complete();
										});
									}),
									o.addEventListener("error", (e) => {
										l(),
											t.error(
												new C({
													error: e,
													status: 0,
													statusText: "JSONP Error",
													url: r,
												}),
											);
									}),
									this.document.body.appendChild(o),
									t.next({ type: x.Sent }),
									() => {
										!s && this.removeListeners(o), l();
									}
								);
							});
						}
						removeListeners(e) {
							!r && (r = this.document.implementation.createHTMLDocument()),
								r.adoptNode(e);
						}
						constructor(e, t) {
							(this.callbackMap = e),
								(this.document = t),
								(this.resolvedPromise = Promise.resolve());
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(o.ɵɵinject(N), o.ɵɵinject(l.DOCUMENT));
						}),
						(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				function V(e, t) {
					return "JSONP" === e.method ? (0, o.inject)($).handle(e) : t(e);
				}
				"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							intercept(e, t) {
								return this.injector.runInContext(() =>
									V(e, (e) => t.handle(e)),
								);
							}
							constructor(e) {
								this.injector = e;
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(o.ɵɵinject(o.EnvironmentInjector));
						}),
							(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac }));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let B = /^\)\]\}',?\n/,
					U = (() => {
						class e {
							handle(e) {
								if ("JSONP" === e.method)
									throw Error(
										"Attempted to construct Jsonp request without HttpClientJsonpModule installed.",
									);
								return new i.Observable((t) => {
									let n;
									let r = this.xhrFactory.build();
									if (
										(r.open(e.method, e.urlWithParams),
										e.withCredentials && (r.withCredentials = !0),
										e.headers.forEach((e, t) =>
											r.setRequestHeader(e, t.join(",")),
										),
										!e.headers.has("Accept") &&
											r.setRequestHeader(
												"Accept",
												"application/json, text/plain, */*",
											),
										!e.headers.has("Content-Type"))
									) {
										let t = e.detectContentTypeHeader();
										null !== t && r.setRequestHeader("Content-Type", t);
									}
									if (e.responseType) {
										let t = e.responseType.toLowerCase();
										r.responseType = "json" !== t ? t : "text";
									}
									let o = e.serializeBody(),
										i = null,
										s = () => {
											var t;
											if (null !== i) return i;
											let n = r.statusText || "OK",
												o = new d(r.getAllResponseHeaders());
											let s =
												("responseURL" in (t = r) && t.responseURL
													? t.responseURL
													: /^X-Request-URL:/m.test(t.getAllResponseHeaders())
													? t.getResponseHeader("X-Request-URL")
													: null) || e.url;
											return (i = new w({
												headers: o,
												status: r.status,
												statusText: n,
												url: s,
											}));
										},
										l = () => {
											let {
													headers: n,
													status: o,
													statusText: i,
													url: l,
												} = s(),
												a = null;
											204 !== o &&
												(a =
													void 0 === r.response ? r.responseText : r.response),
												0 === o && (o = a ? 200 : 0);
											let u = o >= 200 && o < 300;
											if ("json" === e.responseType && "string" == typeof a) {
												let e = a;
												a = a.replace(B, "");
												try {
													a = "" !== a ? JSON.parse(a) : null;
												} catch (t) {
													(a = e), u && ((u = !1), (a = { error: t, text: a }));
												}
											}
											u
												? (t.next(
														new M({
															body: a,
															headers: n,
															status: o,
															statusText: i,
															url: l || void 0,
														}),
												  ),
												  t.complete())
												: t.error(
														new C({
															error: a,
															headers: n,
															status: o,
															statusText: i,
															url: l || void 0,
														}),
												  );
										},
										a = (e) => {
											let { url: n } = s(),
												o = new C({
													error: e,
													status: r.status || 0,
													statusText: r.statusText || "Unknown Error",
													url: n || void 0,
												});
											t.error(o);
										},
										u = !1,
										c = (n) => {
											!u && (t.next(s()), (u = !0));
											let o = { type: x.DownloadProgress, loaded: n.loaded };
											n.lengthComputable && (o.total = n.total),
												"text" === e.responseType &&
													r.responseText &&
													(o.partialText = r.responseText),
												t.next(o);
										},
										f = (e) => {
											let n = { type: x.UploadProgress, loaded: e.loaded };
											e.lengthComputable && (n.total = e.total), t.next(n);
										};
									r.addEventListener("load", l),
										r.addEventListener("error", a),
										r.addEventListener("timeout", a),
										r.addEventListener("abort", a),
										e.reportProgress &&
											(r.addEventListener("progress", c),
											null !== o &&
												r.upload &&
												r.upload.addEventListener("progress", f));
									let h = () => {
											null != n ||
												(n = (function () {
													let e = setTimeout(() => void 0, 2147483647);
													return () => clearTimeout(e);
												})());
										},
										p = () => {
											null == n || n();
										};
									return (
										r.addEventListener("loadstart", h),
										r.addEventListener("loadend", p),
										r.send(o),
										t.next({ type: x.Sent }),
										() => {
											r.removeEventListener("loadstart", h),
												r.removeEventListener("loadend", p),
												r.removeEventListener("error", a),
												r.removeEventListener("abort", a),
												r.removeEventListener("load", l),
												r.removeEventListener("timeout", a),
												null == n || n(),
												e.reportProgress &&
													(r.removeEventListener("progress", c),
													null !== o &&
														r.upload &&
														r.upload.removeEventListener("progress", f)),
												r.readyState !== r.DONE && r.abort();
										}
									);
								});
							}
							constructor(e) {
								this.xhrFactory = e;
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(o.ɵɵinject(l.XhrFactory));
							}),
							(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let H = new o.InjectionToken("XSRF_ENABLED"),
					z = "XSRF-TOKEN",
					W = new o.InjectionToken("XSRF_COOKIE_NAME", {
						providedIn: "root",
						factory: () => z,
					}),
					q = "X-XSRF-TOKEN",
					G = new o.InjectionToken("XSRF_HEADER_NAME", {
						providedIn: "root",
						factory: () => q,
					});
				class Z {}
				let Y = (() => {
					class e {
						getToken() {
							if ("server" === this.platform) return null;
							let e = this.doc.cookie || "";
							return (
								e !== this.lastCookieString &&
									(this.parseCount++,
									(this.lastToken = (0, l.ɵparseCookieValue)(
										e,
										this.cookieName,
									)),
									(this.lastCookieString = e)),
								this.lastToken
							);
						}
						constructor(e, t, n) {
							(this.doc = e),
								(this.platform = t),
								(this.cookieName = n),
								(this.lastCookieString = ""),
								(this.lastToken = null),
								(this.parseCount = 0);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								o.ɵɵinject(l.DOCUMENT),
								o.ɵɵinject(o.PLATFORM_ID),
								o.ɵɵinject(W),
							);
						}),
						(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				function Q(e, t) {
					let n = e.url.toLowerCase();
					if (
						!(0, o.inject)(H) ||
						"GET" === e.method ||
						"HEAD" === e.method ||
						n.startsWith("http://") ||
						n.startsWith("https://")
					)
						return t(e);
					let r = (0, o.inject)(Z).getToken(),
						i = (0, o.inject)(G);
					return (
						null != r &&
							!e.headers.has(i) &&
							(e = e.clone({ headers: e.headers.set(i, r) })),
						t(e)
					);
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let K = (() => {
					class e {
						intercept(e, t) {
							return this.injector.runInContext(() => Q(e, (e) => t.handle(e)));
						}
						constructor(e) {
							this.injector = e;
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(o.ɵɵinject(o.EnvironmentInjector));
						}),
						(e.ɵprov = o.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				var J =
					(((J = J || {})[(J.Interceptors = 0)] = "Interceptors"),
					(J[(J.LegacyInterceptors = 1)] = "LegacyInterceptors"),
					(J[(J.CustomXsrfConfiguration = 2)] = "CustomXsrfConfiguration"),
					(J[(J.NoXsrfProtection = 3)] = "NoXsrfProtection"),
					(J[(J.JsonpSupport = 4)] = "JsonpSupport"),
					(J[(J.RequestsMadeViaParent = 5)] = "RequestsMadeViaParent"),
					J);
				function X(e, t) {
					return { ɵkind: e, ɵproviders: t };
				}
				let ee = new o.InjectionToken("LEGACY_INTERCEPTOR_FN");
				function et({ cookieName: e, headerName: t }) {
					let n = [];
					return (
						void 0 !== e && n.push({ provide: W, useValue: e }),
						void 0 !== t && n.push({ provide: G, useValue: t }),
						X(J.CustomXsrfConfiguration, n)
					);
				}
				(() => {
					class e {
						static disable() {
							return {
								ngModule: e,
								providers: [
									X(J.NoXsrfProtection, [{ provide: H, useValue: !1 }])
										.ɵproviders,
								],
							};
						}
						static withOptions(t = {}) {
							return { ngModule: e, providers: et(t).ɵproviders };
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)();
					}),
						(e.ɵmod = o.ɵɵdefineNgModule({ type: e })),
						(e.ɵinj = o.ɵɵdefineInjector({
							providers: [
								K,
								{ provide: I, useExisting: K, multi: !0 },
								{ provide: Z, useClass: Y },
								et({ cookieName: z, headerName: q }).ɵproviders,
								{ provide: H, useValue: !0 },
							],
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵmod = o.ɵɵdefineNgModule({ type: e })),
							(e.ɵinj = o.ɵɵdefineInjector({
								providers: [
									(function (...e) {
										if (ngDevMode) {
											let t = new Set(e.map((e) => e.ɵkind));
											if (
												t.has(J.NoXsrfProtection) &&
												t.has(J.CustomXsrfConfiguration)
											)
												throw Error(
													ngDevMode
														? "Configuration error: found both withXsrfConfiguration() and withNoXsrfProtection() in the same call to provideHttpClient(), which is a contradiction."
														: "",
												);
										}
										let t = [
											E,
											U,
											F,
											{ provide: a, useExisting: F },
											{ provide: u, useExisting: U },
											{ provide: P, useValue: Q, multi: !0 },
											{ provide: H, useValue: !0 },
											{ provide: Z, useClass: Y },
										];
										for (let n of e) t.push(...n.ɵproviders);
										return (0, o.makeEnvironmentProviders)(t);
									})(
										X(J.LegacyInterceptors, [
											{ provide: ee, useFactory: k },
											{ provide: P, useExisting: ee, multi: !0 },
										]),
									),
								],
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵmod = o.ɵɵdefineNgModule({ type: e })),
							(e.ɵinj = o.ɵɵdefineInjector({
								providers: [
									X(J.JsonpSupport, [
										$,
										{ provide: N, useFactory: L },
										{ provide: P, useValue: V, multi: !0 },
									]).ɵproviders,
								],
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					new o.InjectionToken(
						ngDevMode ? "HTTP_TRANSFER_STATE_CACHE_STATE" : "",
					);
			},
			"../../node_modules/@angular/core/fesm2022/core.mjs": function (e, t, n) {
				"use strict";
				let r, o, i, s, l, a, u, d, c, f;
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					APP_BOOTSTRAP_LISTENER: function () {
						return hb;
					},
					APP_ID: function () {
						return ss;
					},
					APP_INITIALIZER: function () {
						return fK;
					},
					ApplicationModule: function () {
						return hQ;
					},
					ApplicationRef: function () {
						return hw;
					},
					CSP_NONCE: function () {
						return sd;
					},
					ChangeDetectorRef: function () {
						return hI;
					},
					Compiler: function () {
						return f3;
					},
					DEFAULT_CURRENCY_CODE: function () {
						return f1;
					},
					ENVIRONMENT_INITIALIZER: function () {
						return iG;
					},
					ElementRef: function () {
						return sw;
					},
					EnvironmentInjector: function () {
						return st;
					},
					ErrorHandler: function () {
						return sk;
					},
					EventEmitter: function () {
						return c0;
					},
					InjectFlags: function () {
						return ea;
					},
					InjectionToken: function () {
						return iq;
					},
					Injector: function () {
						return s3;
					},
					IterableDiffers: function () {
						return hz;
					},
					KeyValueDiffers: function () {
						return hq;
					},
					LOCALE_ID: function () {
						return f0;
					},
					NgModuleFactory: function () {
						return cu;
					},
					NgModuleRef: function () {
						return ca;
					},
					NgProbeToken: function () {
						return h_;
					},
					NgZone: function () {
						return hr;
					},
					Optional: function () {
						return rV;
					},
					PLATFORM_ID: function () {
						return su;
					},
					PLATFORM_INITIALIZER: function () {
						return sa;
					},
					Renderer2: function () {
						return sS;
					},
					RendererFactory2: function () {
						return sC;
					},
					RendererStyleFlags2: function () {
						return r9;
					},
					SecurityContext: function () {
						return iR;
					},
					SkipSelf: function () {
						return rU;
					},
					TemplateRef: function () {
						return c2;
					},
					Testability: function () {
						return hp;
					},
					TestabilityRegistry: function () {
						return hm;
					},
					TransferState: function () {
						return sh;
					},
					Version: function () {
						return sO;
					},
					ViewContainerRef: function () {
						return fe;
					},
					ViewEncapsulation: function () {
						return eE;
					},
					createEnvironmentInjector: function () {
						return cp;
					},
					createNgModule: function () {
						return cd;
					},
					createPlatformFactory: function () {
						return hj;
					},
					inject: function () {
						return ex;
					},
					isStandalone: function () {
						return e9;
					},
					makeEnvironmentProviders: function () {
						return iK;
					},
					makeStateKey: function () {
						return sc;
					},
					platformCore: function () {
						return hY;
					},
					reflectComponentType: function () {
						return hX;
					},
					ɵConsole: function () {
						return fX;
					},
					ɵINJECTOR_SCOPE: function () {
						return i8;
					},
					ɵInitialRenderPendingTasks: function () {
						return hK;
					},
					ɵLocaleDataIndex: function () {
						return dl;
					},
					ɵRuntimeError: function () {
						return O;
					},
					ɵTESTABILITY: function () {
						return hf;
					},
					ɵTESTABILITY_GETTER: function () {
						return hh;
					},
					ɵXSS_SECURITY_URL: function () {
						return E;
					},
					ɵ_sanitizeHtml: function () {
						return ik;
					},
					ɵ_sanitizeUrl: function () {
						return im;
					},
					ɵallowSanitizationBypassAndThrow: function () {
						return io;
					},
					ɵbypassSanitizationTrustHtml: function () {
						return ii;
					},
					ɵbypassSanitizationTrustResourceUrl: function () {
						return iu;
					},
					ɵbypassSanitizationTrustScript: function () {
						return il;
					},
					ɵbypassSanitizationTrustStyle: function () {
						return is;
					},
					ɵbypassSanitizationTrustUrl: function () {
						return ia;
					},
					ɵcoerceToBoolean: function () {
						return hJ;
					},
					ɵfindLocaleData: function () {
						return dr;
					},
					ɵformatRuntimeError: function () {
						return A;
					},
					ɵgetLocalePluralCase: function () {
						return di;
					},
					ɵglobal: function () {
						return ec;
					},
					ɵisInjectable: function () {
						return et;
					},
					ɵisNgModule: function () {
						return fM;
					},
					ɵisPromise: function () {
						return aT;
					},
					ɵisSubscribable: function () {
						return ak;
					},
					ɵsetDocument: function () {
						return o0;
					},
					ɵstringify: function () {
						return j;
					},
					ɵunwrapSafeValue: function () {
						return ir;
					},
					ɵɵNgOnChangesFeature: function () {
						return tF;
					},
					ɵɵStandaloneFeature: function () {
						return cg;
					},
					ɵɵadvance: function () {
						return sY;
					},
					ɵɵattribute: function () {
						return ae;
					},
					ɵɵcontentQuery: function () {
						return fg;
					},
					ɵɵdefineComponent: function () {
						return eK;
					},
					ɵɵdefineDirective: function () {
						return e3;
					},
					ɵɵdefineInjectable: function () {
						return J;
					},
					ɵɵdefineInjector: function () {
						return X;
					},
					ɵɵdefineNgModule: function () {
						return e1;
					},
					ɵɵdefinePipe: function () {
						return e4;
					},
					ɵɵdirectiveInject: function () {
						return s4;
					},
					ɵɵelement: function () {
						return aC;
					},
					ɵɵelementEnd: function () {
						return aM;
					},
					ɵɵelementStart: function () {
						return aw;
					},
					ɵɵgetCurrentView: function () {
						return aP;
					},
					ɵɵgetInheritedFactory: function () {
						return ro;
					},
					ɵɵinject: function () {
						return e_;
					},
					ɵɵinjectAttribute: function () {
						return rl;
					},
					ɵɵinvalidFactory: function () {
						return s6;
					},
					ɵɵlistener: function () {
						return aF;
					},
					ɵɵloadQuery: function () {
						return fv;
					},
					ɵɵnamespaceHTML: function () {
						return nP;
					},
					ɵɵnamespaceSVG: function () {
						return nA;
					},
					ɵɵproperty: function () {
						return ax;
					},
					ɵɵqueryRefresh: function () {
						return fp;
					},
					ɵɵreference: function () {
						return aj;
					},
					ɵɵresetView: function () {
						return nn;
					},
					ɵɵrestoreView: function () {
						return nt;
					},
					ɵɵsanitizeUrlOrResourceUrl: function () {
						return iz;
					},
					ɵɵstyleProp: function () {
						return ui;
					},
					ɵɵtemplate: function () {
						return ab;
					},
					ɵɵtext: function () {
						return uD;
					},
					ɵɵtextInterpolate1: function () {
						return uC;
					},
				});
				var h,
					p,
					m = n("../../node_modules/@swc/helpers/esm/_object_spread.js"),
					g = n("../../node_modules/@swc/helpers/esm/_object_spread_props.js"),
					v = n("../../node_modules/rxjs/dist/esm5/index.js"),
					y = n("../../node_modules/rxjs/dist/esm5/operators/index.js");
				function b(e) {
					for (let t in e) if (e[t] === b) return t;
					throw Error("Could not find renamed property on target object.");
				}
				function _(e, t) {
					for (let n in t)
						t.hasOwnProperty(n) && !e.hasOwnProperty(n) && (e[n] = t[n]);
				}
				function j(e) {
					if ("string" == typeof e) return e;
					if (Array.isArray(e)) return "[" + e.map(j).join(", ") + "]";
					if (null == e) return "" + e;
					if (e.overriddenName) return `${e.overriddenName}`;
					if (e.name) return `${e.name}`;
					let t = e.toString();
					if (null == t) return "" + t;
					let n = t.indexOf("\n");
					return -1 === n ? t : t.substring(0, n);
				}
				function x(e, t) {
					return null == e || "" === e
						? null === t
							? ""
							: t
						: null == t || "" === t
						? e
						: e + " " + t;
				}
				let D = b({ __forward_ref__: b });
				function w(e) {
					return (
						(e.__forward_ref__ = w),
						(e.toString = function () {
							return j(this());
						}),
						e
					);
				}
				function M(e) {
					return C(e) ? e() : e;
				}
				function C(e) {
					return (
						"function" == typeof e &&
						e.hasOwnProperty(D) &&
						e.__forward_ref__ === w
					);
				}
				function S(e) {
					return e && !!e.ɵproviders;
				}
				let E = "https://g.co/ng/security#xss";
				class O extends Error {
					constructor(e, t) {
						super(A(e, t)), (this.code = e);
					}
				}
				function A(e, t) {
					let n = `NG0${Math.abs(e)}`,
						r = `${n}${t ? ": " + t : ""}`;
					if (ngDevMode && e < 0) {
						let e = !r.match(/[.,;!?\n]$/);
						r = `${r}${
							e ? "." : ""
						} Find more at https://angular.io/errors/${n}`;
					}
					return r;
				}
				function I(e) {
					return "string" == typeof e ? e : null == e ? "" : String(e);
				}
				function P(e) {
					return "function" == typeof e
						? e.name || e.toString()
						: "object" == typeof e && null != e && "function" == typeof e.type
						? e.type.name || e.type.toString()
						: I(e);
				}
				function T(e, t) {
					let n = t ? `. Dependency path: ${t.join(" > ")} > ${e}` : "";
					throw new O(-200, `Circular dependency in DI detected for ${e}${n}`);
				}
				function k() {
					throw Error("Cannot mix multi providers and regular providers");
				}
				function F(e, t, n) {
					if (e && t) {
						let r = t.map((e) => (e == n ? "?" + n + "?" : "..."));
						throw Error(
							`Invalid provider for the NgModule '${j(
								e,
							)}' - only instances of Provider and Type are allowed, got: [${r.join(
								", ",
							)}]`,
						);
					}
					if (S(n)) {
						if (n.ɵfromNgModule)
							throw new O(
								207,
								"Invalid providers from 'importProvidersFrom' present in a non-environment injector. 'importProvidersFrom' can't be used for component providers.",
							);
						throw new O(
							207,
							"Invalid providers present in a non-environment injector. 'EnvironmentProviders' can't be used for component providers.",
						);
					}
					throw Error("Invalid provider");
				}
				function R(e, t) {
					let n = t ? ` in ${t}` : "";
					throw new O(-201, ngDevMode && `No provider for ${P(e)} found${n}`);
				}
				function N(e, t) {
					"number" != typeof e && Y(t, typeof e, "number", "===");
				}
				function L(e, t, n) {
					N(e, "Expected a number"),
						W(e, n, "Expected number to be less than or equal to"),
						G(e, t, "Expected number to be greater than or equal to");
				}
				function $(e, t) {
					"string" != typeof e &&
						Y(t, null === e ? "null" : typeof e, "string", "===");
				}
				function V(e, t, n) {
					e != t && Y(n, e, t, "==");
				}
				function B(e, t, n) {
					!(e != t) && Y(n, e, t, "!=");
				}
				function U(e, t, n) {
					e !== t && Y(n, e, t, "===");
				}
				function H(e, t, n) {
					!(e !== t) && Y(n, e, t, "!==");
				}
				function z(e, t, n) {
					!(e < t) && Y(n, e, t, "<");
				}
				function W(e, t, n) {
					!(e <= t) && Y(n, e, t, "<=");
				}
				function q(e, t, n) {
					!(e > t) && Y(n, e, t, ">");
				}
				function G(e, t, n) {
					!(e >= t) && Y(n, e, t, ">=");
				}
				function Z(e, t) {
					null == e && Y(t, e, null, "!=");
				}
				function Y(e, t, n, r) {
					throw Error(
						`ASSERTION ERROR: ${e}` +
							(null == r ? "" : ` [Expected=> ${n} ${r} ${t} <=Actual]`),
					);
				}
				function Q(e) {
					!("undefined" != typeof Node && e instanceof Node) &&
						!(
							"object" == typeof e &&
							null != e &&
							"WebWorkerRenderNode" === e.constructor.name
						) &&
						Y(
							`The provided value must be an instance of a DOM Node but got ${j(
								e,
							)}`,
						);
				}
				function K(e, t) {
					Z(e, "Array must be defined.");
					let n = e.length;
					(t < 0 || t >= n) &&
						Y(`Index expected to be less than ${n} but got ${t}`);
				}
				function J(e) {
					return {
						token: e.token,
						providedIn: e.providedIn || null,
						factory: e.factory,
						value: void 0,
					};
				}
				function X(e) {
					return { providers: e.providers || [], imports: e.imports || [] };
				}
				function ee(e) {
					return en(e, eo) || en(e, es);
				}
				function et(e) {
					return null !== ee(e);
				}
				function en(e, t) {
					return e.hasOwnProperty(t) ? e[t] : null;
				}
				function er(e) {
					return e && (e.hasOwnProperty(ei) || e.hasOwnProperty(el))
						? e[ei]
						: null;
				}
				let eo = b({ ɵprov: b }),
					ei = b({ ɵinj: b }),
					es = b({ ngInjectableDef: b }),
					el = b({ ngInjectorDef: b });
				var ea =
					(((ea = ea || {})[(ea.Default = 0)] = "Default"),
					(ea[(ea.Host = 1)] = "Host"),
					(ea[(ea.Self = 2)] = "Self"),
					(ea[(ea.SkipSelf = 4)] = "SkipSelf"),
					(ea[(ea.Optional = 8)] = "Optional"),
					ea);
				function eu(e) {
					let t = r;
					return (r = e), t;
				}
				function ed(e, t, n) {
					let r = ee(e);
					return r && "root" == r.providedIn
						? void 0 === r.value
							? (r.value = r.factory())
							: r.value
						: n & ea.Optional
						? null
						: void 0 !== t
						? t
						: void R(j(e), "Injector");
				}
				let ec =
					("undefined" != typeof globalThis && globalThis) ||
					("undefined" != typeof global && global) ||
					("undefined" != typeof window && window) ||
					("undefined" != typeof self &&
						"undefined" != typeof WorkerGlobalScope &&
						self instanceof WorkerGlobalScope &&
						self);
				function ef() {
					return (
						!!("undefined" == typeof ngDevMode || ngDevMode) &&
						("object" != typeof ngDevMode &&
							!(function () {
								let e =
										"undefined" != typeof location ? location.toString() : "",
									t = {
										namedConstructors:
											-1 != e.indexOf("ngDevMode=namedConstructors"),
										firstCreatePass: 0,
										tNode: 0,
										tView: 0,
										rendererCreateTextNode: 0,
										rendererSetText: 0,
										rendererCreateElement: 0,
										rendererAddEventListener: 0,
										rendererSetAttribute: 0,
										rendererRemoveAttribute: 0,
										rendererSetProperty: 0,
										rendererSetClassName: 0,
										rendererAddClass: 0,
										rendererRemoveClass: 0,
										rendererSetStyle: 0,
										rendererRemoveStyle: 0,
										rendererDestroy: 0,
										rendererDestroyNode: 0,
										rendererMoveNode: 0,
										rendererRemoveNode: 0,
										rendererAppendChild: 0,
										rendererInsertBefore: 0,
										rendererCreateComment: 0,
										hydratedNodes: 0,
										hydratedComponents: 0,
										dehydratedViewsRemoved: 0,
										dehydratedViewsCleanupRuns: 0,
										componentsSkippedHydration: 0,
									},
									n = -1 === e.indexOf("ngDevMode=false");
								ec.ngDevMode = n && t;
							})(),
						"undefined" != typeof ngDevMode && !!ngDevMode)
					);
				}
				let eh = {},
					ep = "__NG_DI_FLAG__",
					em = "ngTempTokenPath",
					eg = /\n/gm,
					ev = "__source";
				function ey(e) {
					let t = d;
					return (d = e), t;
				}
				function eb(e, t = ea.Default) {
					if (void 0 === d)
						throw new O(
							-203,
							ngDevMode &&
								"inject() must be called from an injection context such as a constructor, a factory function, a field initializer, or a function used with `runInInjectionContext`.",
						);
					if (null === d) return ed(e, void 0, t);
					return d.get(e, t & ea.Optional ? null : void 0, t);
				}
				function e_(e, t = ea.Default) {
					return (r || eb)(M(e), t);
				}
				function ej(e) {
					throw new O(
						202,
						ngDevMode &&
							`This constructor is not compatible with Angular Dependency Injection because its dependency at index ${e} of the parameter list is invalid.
This can happen if the dependency type is a primitive like a string or if an ancestor of this class is missing an Angular decorator.

Please check that 1) the type for the parameter at index ${e} is correct and 2) the correct Angular decorators are defined for this class and its ancestors.`,
					);
				}
				function ex(e, t = ea.Default) {
					return e_(e, eD(t));
				}
				function eD(e) {
					return void 0 === e || "number" == typeof e
						? e
						: 0 |
								(e.optional && 8) |
								(e.host && 1) |
								(e.self && 2) |
								(e.skipSelf && 4);
				}
				function ew(e) {
					let t = [];
					for (let n = 0; n < e.length; n++) {
						let r = M(e[n]);
						if (Array.isArray(r)) {
							let e;
							if (0 === r.length)
								throw new O(
									900,
									ngDevMode && "Arguments array must have arguments.",
								);
							let n = ea.Default;
							for (let t = 0; t < r.length; t++) {
								let o = r[t],
									i = (function (e) {
										return e[ep];
									})(o);
								"number" == typeof i
									? -1 === i
										? (e = o.token)
										: (n |= i)
									: (e = o);
							}
							t.push(e_(e, n));
						} else t.push(e_(r));
					}
					return t;
				}
				function eM(e, t) {
					return (e[ep] = t), (e.prototype[ep] = t), e;
				}
				function eC(e) {
					return { toString: e }.toString();
				}
				var eS =
					(((eS = eS || {})[(eS.OnPush = 0)] = "OnPush"),
					(eS[(eS.Default = 1)] = "Default"),
					eS);
				var eE =
					(((h = eE || (eE = {}))[(h.Emulated = 0)] = "Emulated"),
					(h[(h.None = 2)] = "None"),
					(h[(h.ShadowDom = 3)] = "ShadowDom"),
					eE);
				let eO = {},
					eA = [];
				("undefined" == typeof ngDevMode || ngDevMode) &&
					ef() &&
					(Object.freeze(eO), Object.freeze(eA));
				let eI = b({ ɵcmp: b }),
					eP = b({ ɵdir: b }),
					eT = b({ ɵpipe: b }),
					ek = b({ ɵmod: b }),
					eF = b({ ɵfac: b }),
					eR = b({ __NG_ELEMENT_ID__: b }),
					eN = b({ __NG_ENV_ID__: b });
				function eL(e, t, n) {
					ngDevMode && B(t, "", 'can not look for "" string.');
					let r = e.length;
					for (;;) {
						let o = e.indexOf(t, n);
						if (-1 === o) return o;
						if (0 === o || 32 >= e.charCodeAt(o - 1)) {
							let n = t.length;
							if (o + n === r || 32 >= e.charCodeAt(o + n)) return o;
						}
						n = o + 1;
					}
				}
				function e$(e, t, n) {
					let r = 0;
					for (; r < n.length; ) {
						let o = n[r];
						if ("number" == typeof o) {
							if (0 !== o) break;
							r++;
							let i = n[r++],
								s = n[r++],
								l = n[r++];
							ngDevMode && ngDevMode.rendererSetAttribute++,
								e.setAttribute(t, s, l, i);
						} else {
							let i = n[++r];
							ngDevMode && ngDevMode.rendererSetAttribute++,
								eB(o) ? e.setProperty(t, o, i) : e.setAttribute(t, o, i),
								r++;
						}
					}
					return r;
				}
				function eV(e) {
					return 3 === e || 4 === e || 6 === e;
				}
				function eB(e) {
					return 64 === e.charCodeAt(0);
				}
				function eU(e, t) {
					if (null === t || 0 === t.length);
					else if (null === e || 0 === e.length) e = t.slice();
					else {
						let n = -1;
						for (let r = 0; r < t.length; r++) {
							let o = t[r];
							"number" == typeof o
								? (n = o)
								: 0 === n ||
								  (-1 === n || 2 === n
										? eH(e, n, o, null, t[++r])
										: eH(e, n, o, null, null));
						}
					}
					return e;
				}
				function eH(e, t, n, r, o) {
					let i = 0,
						s = e.length;
					if (-1 === t) s = -1;
					else
						for (; i < e.length; ) {
							let n = e[i++];
							if ("number" == typeof n) {
								if (n === t) {
									s = -1;
									break;
								}
								if (n > t) {
									s = i - 1;
									break;
								}
							}
						}
					for (; i < e.length; ) {
						let t = e[i];
						if ("number" == typeof t) break;
						if (t === n) {
							if (null === r) {
								null !== o && (e[i + 1] = o);
								return;
							}
							if (r === e[i + 1]) {
								e[i + 2] = o;
								return;
							}
						}
						i++, null !== r && i++, null !== o && i++;
					}
					-1 !== s && (e.splice(s, 0, t), (i = s + 1)),
						e.splice(i++, 0, n),
						null !== r && e.splice(i++, 0, r),
						null !== o && e.splice(i++, 0, o);
				}
				let ez = "ng-template";
				function eW(e) {
					return 4 === e.type && e.value !== ez;
				}
				function eq(e) {
					return (1 & e) == 0;
				}
				function eG(e, t, n = !1) {
					for (let r = 0; r < t.length; r++)
						if (
							(function (e, t, n) {
								ngDevMode && Z(t[0], "Selector should have a tag name");
								let r = 4,
									o = e.attrs || [],
									i = (function (e) {
										for (let t = 0; t < e.length; t++) {
											let n = e[t];
											if (eV(n)) return t;
										}
										return e.length;
									})(o),
									s = !1;
								for (let l = 0; l < t.length; l++) {
									let a = t[l];
									if ("number" == typeof a) {
										if (!s && !eq(r) && !eq(a)) return !1;
										if (s && eq(a)) continue;
										(s = !1), (r = a | (1 & r));
										continue;
									}
									if (!s) {
										if (4 & r) {
											if (
												((r = 2 | (1 & r)),
												("" !== a &&
													!(function (e, t, n) {
														let r = 4 !== e.type || n ? e.value : ez;
														return t === r;
													})(e, a, n)) ||
													("" === a && 1 === t.length))
											) {
												if (eq(r)) return !1;
												s = !0;
											}
										} else {
											let u = 8 & r ? a : t[++l];
											if (8 & r && null !== e.attrs) {
												if (
													!(function (e, t, n) {
														ngDevMode &&
															V(
																t,
																t.toLowerCase(),
																"Class name expected to be lowercase.",
															);
														let r = 0,
															o = !0;
														for (; r < e.length; ) {
															let i = e[r++];
															if ("string" == typeof i && o) {
																let o = e[r++];
																if (
																	n &&
																	"class" === i &&
																	-1 !== eL(o.toLowerCase(), t, 0)
																)
																	return !0;
															} else if (1 === i) {
																for (
																	;
																	r < e.length &&
																	"string" == typeof (i = e[r++]);
																)
																	if (i.toLowerCase() === t) return !0;
																break;
															} else "number" == typeof i && (o = !1);
														}
														return !1;
													})(e.attrs, u, n)
												) {
													if (eq(r)) return !1;
													s = !0;
												}
												continue;
											}
											let d = 8 & r ? "class" : a,
												c = (function (e, t, n, r) {
													if (null === t) return -1;
													let o = 0;
													if (!r && n)
														return (function (e, t) {
															let n = e.indexOf(4);
															if (n > -1)
																for (n++; n < e.length; ) {
																	let r = e[n];
																	if ("number" == typeof r) break;
																	if (r === t) return n;
																	n++;
																}
															return -1;
														})(t, e);
													{
														let n = !1;
														for (; o < t.length; ) {
															let r = t[o];
															if (r === e) return o;
															if (3 === r || 6 === r) n = !0;
															else if (1 === r || 2 === r) {
																let e = t[++o];
																for (; "string" == typeof e; ) e = t[++o];
																continue;
															} else if (4 === r) break;
															else if (0 === r) {
																o += 4;
																continue;
															}
															o += n ? 1 : 2;
														}
														return -1;
													}
												})(d, o, eW(e), n);
											if (-1 === c) {
												if (eq(r)) return !1;
												s = !0;
												continue;
											}
											if ("" !== u) {
												let e;
												c > i
													? (e = "")
													: (ngDevMode &&
															B(
																o[c],
																0,
																"We do not match directives on namespaced attributes",
															),
													  (e = o[c + 1].toLowerCase()));
												let t = 8 & r ? e : null;
												if ((t && -1 !== eL(t, u, 0)) || (2 & r && u !== e)) {
													if (eq(r)) return !1;
													s = !0;
												}
											}
										}
									}
								}
								return eq(r) || s;
							})(e, t[r], n)
						)
							return !0;
					return !1;
				}
				function eZ(e, t) {
					return e ? ":not(" + t.trim() + ")" : t;
				}
				function eY(e) {
					let t = e[0],
						n = 1,
						r = 2,
						o = "",
						i = !1;
					for (; n < e.length; ) {
						let s = e[n];
						if ("string" == typeof s) {
							if (2 & r) {
								let t = e[++n];
								o += "[" + s + (t.length > 0 ? '="' + t + '"' : "") + "]";
							} else 8 & r ? (o += "." + s) : 4 & r && (o += " " + s);
						} else
							"" !== o && !eq(s) && ((t += eZ(i, o)), (o = "")),
								(r = s),
								(i = i || !eq(r));
						n++;
					}
					return "" !== o && (t += eZ(i, o)), t;
				}
				function eQ(e) {
					return e.map(eY).join(",");
				}
				function eK(e) {
					return eC(() => {
						("undefined" == typeof ngDevMode || ngDevMode) && ef();
						let t = tt(e),
							n = g._(m._({}, t), {
								decls: e.decls,
								vars: e.vars,
								template: e.template,
								consts: e.consts || null,
								ngContentSelectors: e.ngContentSelectors,
								onPush: e.changeDetection === eS.OnPush,
								directiveDefs: null,
								pipeDefs: null,
								dependencies: (t.standalone && e.dependencies) || null,
								getStandaloneInjector: null,
								data: e.data || {},
								encapsulation: e.encapsulation || eE.Emulated,
								styles: e.styles || eA,
								_: null,
								schemas: e.schemas || null,
								tView: null,
								id: "",
							});
						tn(n);
						let r = e.dependencies;
						return (
							(n.directiveDefs = tr(r, !1)),
							(n.pipeDefs = tr(r, !0)),
							(n.id = (function (e) {
								let t = 0,
									n = [
										e.selectors,
										e.ngContentSelectors,
										e.hostVars,
										e.hostAttrs,
										e.consts,
										e.vars,
										e.decls,
										e.encapsulation,
										e.standalone,
										Object.getOwnPropertyNames(e.type.prototype),
										!!e.contentQueries,
										!!e.viewQuery,
									].join("|");
								for (let e of n) t = (Math.imul(31, t) + e.charCodeAt(0)) << 0;
								t += 2147483648;
								let r = "c" + t;
								if ("undefined" == typeof ngDevMode || ngDevMode) {
									if (to.has(r)) {
										let t = to.get(r);
										t !== e.type &&
											console.warn(
												A(
													-912,
													`Component ID generation collision detected. Components '${
														t.name
													}' and '${e.type.name}' with selector '${eQ(
														e.selectors,
													)}' generated the same component ID. To fix this, you can change the selector of one of those components or add an extra host attribute to force a different ID.`,
												),
											);
									} else to.set(r, e.type);
								}
								return r;
							})(n)),
							n
						);
					});
				}
				function eJ(e, t, n) {
					let r = e.ɵcmp;
					(r.directiveDefs = tr(t, !1)), (r.pipeDefs = tr(n, !0));
				}
				function eX(e) {
					return e6(e) || e8(e);
				}
				function e0(e) {
					return null !== e;
				}
				function e1(e) {
					return eC(() => {
						let t = {
							type: e.type,
							bootstrap: e.bootstrap || eA,
							declarations: e.declarations || eA,
							imports: e.imports || eA,
							exports: e.exports || eA,
							transitiveCompileScopes: null,
							schemas: e.schemas || null,
							id: e.id || null,
						};
						return t;
					});
				}
				function e5(e, t) {
					return eC(() => {
						let n = te(e, !0);
						(n.declarations = t.declarations || eA),
							(n.imports = t.imports || eA),
							(n.exports = t.exports || eA);
					});
				}
				function e2(e, t) {
					if (null == e) return eO;
					let n = {};
					for (let r in e)
						if (e.hasOwnProperty(r)) {
							let o = e[r],
								i = o;
							Array.isArray(o) && ((i = o[1]), (o = o[0])),
								(n[o] = r),
								t && (t[o] = i);
						}
					return n;
				}
				function e3(e) {
					return eC(() => {
						let t = tt(e);
						return tn(t), t;
					});
				}
				function e4(e) {
					return {
						type: e.type,
						name: e.name,
						factory: null,
						pure: !1 !== e.pure,
						standalone: !0 === e.standalone,
						onDestroy: e.type.prototype.ngOnDestroy || null,
					};
				}
				function e6(e) {
					return e[eI] || null;
				}
				function e8(e) {
					return e[eP] || null;
				}
				function e7(e) {
					return e[eT] || null;
				}
				function e9(e) {
					let t = e6(e) || e8(e) || e7(e);
					return null !== t && t.standalone;
				}
				function te(e, t) {
					let n = e[ek] || null;
					if (!n && !0 === t)
						throw Error(`Type ${j(e)} does not have 'ɵmod' property.`);
					return n;
				}
				function tt(e) {
					let t = {};
					return {
						type: e.type,
						providersResolver: null,
						factory: null,
						hostBindings: e.hostBindings || null,
						hostVars: e.hostVars || 0,
						hostAttrs: e.hostAttrs || null,
						contentQueries: e.contentQueries || null,
						declaredInputs: t,
						exportAs: e.exportAs || null,
						standalone: !0 === e.standalone,
						selectors: e.selectors || eA,
						viewQuery: e.viewQuery || null,
						features: e.features || null,
						setInput: null,
						findHostDirectiveDefs: null,
						hostDirectives: null,
						inputs: e2(e.inputs, t),
						outputs: e2(e.outputs),
					};
				}
				function tn(e) {
					var t;
					null === (t = e.features) || void 0 === t || t.forEach((t) => t(e));
				}
				function tr(e, t) {
					if (!e) return null;
					let n = t ? e7 : eX;
					return () =>
						("function" == typeof e ? e() : e).map((e) => n(e)).filter(e0);
				}
				let to = new Map();
				function ti(e) {
					return Array.isArray(e) && "object" == typeof e[1];
				}
				function ts(e) {
					return Array.isArray(e) && !0 === e[1];
				}
				function tl(e) {
					return (4 & e.flags) != 0;
				}
				function ta(e) {
					return e.componentOffset > -1;
				}
				function tu(e) {
					return (1 & e.flags) == 1;
				}
				function td(e) {
					return !!e.template;
				}
				function tc(e, t) {
					tf(e, t[1]);
				}
				function tf(e, t) {
					th(e),
						e.hasOwnProperty("tView_") &&
							V(e.tView_, t, "This TNode does not belong to this TView.");
				}
				function th(e) {
					Z(e, "TNode must be defined"),
						!(
							e &&
							"object" == typeof e &&
							e.hasOwnProperty("directiveStylingLast")
						) && Y("Not of type TNode, got: " + e);
				}
				function tp(e) {
					Z(e, "Expected TIcu to be defined"),
						"number" != typeof e.currentCaseLViewIndex &&
							Y("Object is not of TIcu type.");
				}
				function tm(e) {
					Z(e, "currentTNode should exist!"),
						Z(e.parent, "currentTNode should have a parent");
				}
				function tg(e) {
					Z(e, "LContainer must be defined"),
						V(ts(e), !0, "Expecting LContainer");
				}
				function tv(e) {
					e && V(ti(e), !0, "Expecting LView or undefined or null");
				}
				function ty(e) {
					Z(e, "LView must be defined"), V(ti(e), !0, "Expecting LView");
				}
				function tb(e, t) {
					V(
						e.firstCreatePass,
						!0,
						t || "Should only be called in first create pass.",
					);
				}
				function t_(e, t) {
					V(
						e.firstUpdatePass,
						!0,
						t || "Should only be called in first update pass.",
					);
				}
				function tj(e, t) {
					let n = e[1];
					tx(n.expandoStartIndex, e.length, t);
				}
				function tx(e, t, n) {
					!(e <= n && n < t) &&
						Y(`Index out of range (expecting ${e} <= ${n} < ${t})`);
				}
				function tD(e, t) {
					Z(
						e,
						t ||
							"Component views should always have a parent view (component's host view)",
					);
				}
				function tw(e, t) {
					tj(e, t),
						tj(e, t + 8),
						N(e[t + 0], "injectorIndex should point to a bloom filter"),
						N(e[t + 1], "injectorIndex should point to a bloom filter"),
						N(e[t + 2], "injectorIndex should point to a bloom filter"),
						N(e[t + 3], "injectorIndex should point to a bloom filter"),
						N(e[t + 4], "injectorIndex should point to a bloom filter"),
						N(e[t + 5], "injectorIndex should point to a bloom filter"),
						N(e[t + 6], "injectorIndex should point to a bloom filter"),
						N(e[t + 7], "injectorIndex should point to a bloom filter"),
						N(e[t + 8], "injectorIndex should point to parent injector");
				}
				function tM(e, t) {
					let n = e.hasOwnProperty(eF);
					if (!n && !0 === t && ngDevMode)
						throw Error(`Type ${j(e)} does not have 'ɵfac' property.`);
					return n ? e[eF] : null;
				}
				Symbol("SIGNAL");
				let tC =
						null !== (p = ec.WeakRef) && void 0 !== p
							? p
							: class e {
									deref() {
										return this.ref;
									}
									constructor(e) {
										this.ref = e;
									}
							  },
					tS = 0,
					tE = null,
					tO = !1;
				function tA(e) {
					let t = tE;
					return (tE = e), t;
				}
				class tI {
					consumerPollProducersForChange() {
						for (let [e, t] of this.producers) {
							let n = t.producerNode.deref();
							if (
								void 0 === n ||
								t.atTrackingVersion !== this.trackingVersion
							) {
								this.producers.delete(e),
									null == n || n.consumers.delete(this.id);
								continue;
							}
							if (n.producerPollStatus(t.seenValueVersion)) return !0;
						}
						return !1;
					}
					producerMayHaveChanged() {
						let e = tO;
						tO = !0;
						try {
							for (let [e, t] of this.consumers) {
								let n = t.consumerNode.deref();
								if (void 0 === n || n.trackingVersion !== t.atTrackingVersion) {
									this.consumers.delete(e),
										null == n || n.producers.delete(this.id);
									continue;
								}
								n.onConsumerDependencyMayHaveChanged();
							}
						} finally {
							tO = e;
						}
					}
					producerAccessed() {
						if (tO)
							throw Error(
								"undefined" != typeof ngDevMode && ngDevMode
									? "Assertion error: signal read during notification phase"
									: "",
							);
						if (null === tE) return;
						let e = tE.producers.get(this.id);
						void 0 === e
							? ((e = {
									consumerNode: tE.ref,
									producerNode: this.ref,
									seenValueVersion: this.valueVersion,
									atTrackingVersion: tE.trackingVersion,
							  }),
							  tE.producers.set(this.id, e),
							  this.consumers.set(tE.id, e))
							: ((e.seenValueVersion = this.valueVersion),
							  (e.atTrackingVersion = tE.trackingVersion));
					}
					get hasProducers() {
						return this.producers.size > 0;
					}
					get producerUpdatesAllowed() {
						return (null == tE ? void 0 : tE.consumerAllowSignalWrites) !== !1;
					}
					producerPollStatus(e) {
						return (
							this.valueVersion !== e ||
							(this.onProducerUpdateValueVersion(), this.valueVersion !== e)
						);
					}
					constructor() {
						(this.id = tS++),
							(this.ref = (function (e) {
								if (
									"undefined" != typeof ngDevMode &&
									ngDevMode &&
									void 0 === tC
								)
									throw Error(
										"Angular requires a browser which supports the 'WeakRef' API",
									);
								return new tC(e);
							})(this)),
							(this.producers = new Map()),
							(this.consumers = new Map()),
							(this.trackingVersion = 0),
							(this.valueVersion = 0);
					}
				}
				Symbol("UNSET"), Symbol("COMPUTING"), Symbol("ERRORED");
				let tP = () => {};
				class tT extends tI {
					notify() {
						!this.dirty && this.schedule(this), (this.dirty = !0);
					}
					onConsumerDependencyMayHaveChanged() {
						this.notify();
					}
					onProducerUpdateValueVersion() {}
					run() {
						if (
							((this.dirty = !1),
							0 !== this.trackingVersion &&
								!this.consumerPollProducersForChange())
						)
							return;
						let e = tA(this);
						this.trackingVersion++;
						try {
							this.cleanupFn(),
								(this.cleanupFn = tP),
								this.watch(this.registerOnCleanup);
						} finally {
							tA(e);
						}
					}
					cleanup() {
						this.cleanupFn();
					}
					constructor(e, t, n) {
						super(),
							(this.watch = e),
							(this.schedule = t),
							(this.dirty = !1),
							(this.cleanupFn = tP),
							(this.registerOnCleanup = (e) => {
								this.cleanupFn = e;
							}),
							(this.consumerAllowSignalWrites = n);
					}
				}
				class tk {
					isFirstChange() {
						return this.firstChange;
					}
					constructor(e, t, n) {
						(this.previousValue = e),
							(this.currentValue = t),
							(this.firstChange = n);
					}
				}
				function tF() {
					return tR;
				}
				function tR(e) {
					return e.type.prototype.ngOnChanges && (e.setInput = tL), tN;
				}
				function tN() {
					let e = (function (e) {
							return e[t$] || null;
						})(this),
						t = null == e ? void 0 : e.current;
					if (t) {
						let n = e.previous;
						if (n === eO) e.previous = t;
						else for (let e in t) n[e] = t[e];
						(e.current = null), this.ngOnChanges(t);
					}
				}
				function tL(e, t, n, r) {
					let o = this.declaredInputs[n];
					ngDevMode && $(o, "Name of input in ngOnChanges has to be a string");
					let i =
							(function (e) {
								return e[t$] || null;
							})(e) ||
							(function (e, t) {
								return (e[t$] = t);
							})(e, { previous: eO, current: null }),
						s = i.current || (i.current = {}),
						l = i.previous,
						a = l[o];
					(s[o] = new tk(a && a.currentValue, t, l === eO)), (e[r] = t);
				}
				tF.ngInherit = !0;
				let t$ = "__ngSimpleChanges__";
				function tV(e) {
					return e[t$] || null;
				}
				let tB = null,
					tU = (e) => {
						tB = e;
					},
					tH = function (e, t, n) {
						null != tB && tB(e, t, n);
					},
					tz = "math";
				function tW(e) {
					for (; Array.isArray(e); ) e = e[0];
					return e;
				}
				function tq(e, t) {
					return (
						ngDevMode && K(t, e),
						ngDevMode && G(e, 25, "Expected to be past HEADER_OFFSET"),
						tW(t[e])
					);
				}
				function tG(e, t) {
					ngDevMode && tc(e, t), ngDevMode && K(t, e.index);
					let n = tW(t[e.index]);
					return n;
				}
				function tZ(e, t) {
					ngDevMode && q(t, -1, "wrong index for TNode"),
						ngDevMode && z(t, e.data.length, "wrong index for TNode");
					let n = e.data[t];
					return ngDevMode && null !== n && th(n), n;
				}
				function tY(e, t) {
					return ngDevMode && K(e, t), e[t];
				}
				function tQ(e, t) {
					ngDevMode && K(t, e);
					let n = t[e],
						r = ti(n) ? n : n[0];
					return r;
				}
				function tK(e) {
					return (4 & e[2]) == 4;
				}
				function tJ(e) {
					return (128 & e[2]) == 128;
				}
				function tX(e, t) {
					return null == t ? null : (ngDevMode && K(e, t), e[t]);
				}
				function t0(e) {
					e[17] = 0;
				}
				function t1(e) {
					1024 & e[2] && ((e[2] &= -1025), t5(e, -1));
				}
				function t5(e, t) {
					let n = e[3];
					if (null === n) return;
					n[5] += t;
					let r = n;
					for (
						n = n[3];
						null !== n && ((1 === t && 1 === r[5]) || (-1 === t && 0 === r[5]));
					)
						(n[5] += t), (r = n), (n = n[3]);
				}
				function t2(e, t) {
					if ((256 & e[2]) == 256)
						throw new O(911, ngDevMode && "View has already been destroyed.");
					null === e[21] && (e[21] = []), e[21].push(t);
				}
				let t3 = {
						lFrame: nw(null),
						bindingsEnabled: !0,
						skipHydrationRootTNode: null,
					},
					t4 = !1;
				function t6() {
					return t3.bindingsEnabled;
				}
				function t8() {
					t3.bindingsEnabled = !0;
				}
				function t7() {
					t3.bindingsEnabled = !1;
				}
				function t9() {
					return t3.lFrame.lView;
				}
				function ne() {
					return t3.lFrame.tView;
				}
				function nt(e) {
					return (t3.lFrame.contextLView = e), e[8];
				}
				function nn(e) {
					return (t3.lFrame.contextLView = null), e;
				}
				function nr() {
					let e = no();
					for (; null !== e && 64 === e.type; ) e = e.parent;
					return e;
				}
				function no() {
					return t3.lFrame.currentTNode;
				}
				function ni() {
					let e = t3.lFrame,
						t = e.currentTNode;
					return e.isParent ? t : t.parent;
				}
				function ns(e, t) {
					ngDevMode && e && tf(e, t3.lFrame.tView);
					let n = t3.lFrame;
					(n.currentTNode = e), (n.isParent = t);
				}
				function nl() {
					return t3.lFrame.isParent;
				}
				function na() {
					t3.lFrame.isParent = !1;
				}
				function nu() {
					return ngDevMode || Y("Must never be called in production mode"), t4;
				}
				function nd(e) {
					ngDevMode || Y("Must never be called in production mode"), (t4 = e);
				}
				function nc() {
					let e = t3.lFrame,
						t = e.bindingRootIndex;
					return (
						-1 === t && (t = e.bindingRootIndex = e.tView.bindingStartIndex), t
					);
				}
				function nf() {
					return t3.lFrame.bindingIndex;
				}
				function nh(e) {
					return (t3.lFrame.bindingIndex = e);
				}
				function np() {
					return t3.lFrame.bindingIndex++;
				}
				function nm(e) {
					let t = t3.lFrame,
						n = t.bindingIndex;
					return (t.bindingIndex = t.bindingIndex + e), n;
				}
				function ng(e) {
					t3.lFrame.inI18n = e;
				}
				function nv(e) {
					t3.lFrame.currentDirectiveIndex = e;
				}
				function ny(e) {
					let t = t3.lFrame.currentDirectiveIndex;
					return -1 === t ? null : e[t];
				}
				function nb() {
					return t3.lFrame.currentQueryIndex;
				}
				function n_(e) {
					t3.lFrame.currentQueryIndex = e;
				}
				function nj(e, t, n) {
					if ((ngDevMode && tv(e), n & ea.SkipSelf)) {
						ngDevMode && tf(t, e[1]);
						let r = t,
							o = e;
						for (
							;
							ngDevMode && Z(r, "Parent TNode should be defined"),
								null === (r = r.parent) && !(n & ea.Host);
						) {
							if (
								null ===
								(r = (function (e) {
									let t = e[1];
									return 2 === t.type
										? (ngDevMode &&
												Z(
													t.declTNode,
													"Embedded TNodes should have declaration parents.",
												),
										  t.declTNode)
										: 1 === t.type
										? e[6]
										: null;
								})(o))
							)
								break;
							if (
								(ngDevMode && Z(o, "Parent LView should be defined"),
								(o = o[14]),
								10 & r.type)
							)
								break;
						}
						if (null === r) return !1;
						(t = r), (e = o);
					}
					ngDevMode && tc(t, e);
					let r = (t3.lFrame = nD());
					return (r.currentTNode = t), (r.lView = e), !0;
				}
				function nx(e) {
					ngDevMode && B(e[0], e[1], "????"), ngDevMode && tv(e);
					let t = nD();
					ngDevMode &&
						(V(t.isParent, !0, "Expected clean LFrame"),
						V(t.lView, null, "Expected clean LFrame"),
						V(t.tView, null, "Expected clean LFrame"),
						V(t.selectedIndex, -1, "Expected clean LFrame"),
						V(t.elementDepthCount, 0, "Expected clean LFrame"),
						V(t.currentDirectiveIndex, -1, "Expected clean LFrame"),
						V(t.currentNamespace, null, "Expected clean LFrame"),
						V(t.bindingRootIndex, -1, "Expected clean LFrame"),
						V(t.currentQueryIndex, 0, "Expected clean LFrame"));
					let n = e[1];
					(t3.lFrame = t),
						ngDevMode && n.firstChild && tf(n.firstChild, n),
						(t.currentTNode = n.firstChild),
						(t.lView = e),
						(t.tView = n),
						(t.contextLView = e),
						(t.bindingIndex = n.bindingStartIndex),
						(t.inI18n = !1);
				}
				function nD() {
					let e = t3.lFrame,
						t = null === e ? null : e.child,
						n = null === t ? nw(e) : t;
					return n;
				}
				function nw(e) {
					let t = {
						currentTNode: null,
						isParent: !0,
						lView: null,
						tView: null,
						selectedIndex: -1,
						contextLView: null,
						elementDepthCount: 0,
						currentNamespace: null,
						currentDirectiveIndex: -1,
						bindingRootIndex: -1,
						bindingIndex: -1,
						currentQueryIndex: 0,
						parent: e,
						child: null,
						inI18n: !1,
					};
					return null !== e && (e.child = t), t;
				}
				function nM() {
					let e = t3.lFrame;
					return (
						(t3.lFrame = e.parent), (e.currentTNode = null), (e.lView = null), e
					);
				}
				function nC() {
					let e = nM();
					(e.isParent = !0),
						(e.tView = null),
						(e.selectedIndex = -1),
						(e.contextLView = null),
						(e.elementDepthCount = 0),
						(e.currentDirectiveIndex = -1),
						(e.currentNamespace = null),
						(e.bindingRootIndex = -1),
						(e.bindingIndex = -1),
						(e.currentQueryIndex = 0);
				}
				function nS() {
					return t3.lFrame.selectedIndex;
				}
				function nE(e) {
					ngDevMode &&
						-1 !== e &&
						G(e, 25, "Index must be past HEADER_OFFSET (or -1)."),
						ngDevMode &&
							z(
								e,
								t3.lFrame.lView.length,
								"Can't set index passed end of LView",
							),
						(t3.lFrame.selectedIndex = e);
				}
				function nO() {
					let e = t3.lFrame;
					return tZ(e.tView, e.selectedIndex);
				}
				function nA() {
					t3.lFrame.currentNamespace = "svg";
				}
				function nI() {
					t3.lFrame.currentNamespace = tz;
				}
				function nP() {
					(function () {
						t3.lFrame.currentNamespace = null;
					})();
				}
				let nT = !0;
				function nk() {
					return nT;
				}
				function nF(e) {
					nT = e;
				}
				function nR(e, t) {
					ngDevMode && tb(e);
					for (let u = t.directiveStart, d = t.directiveEnd; u < d; u++) {
						var n, r, o, i, s, l, a;
						let t = e.data[u];
						ngDevMode && Z(t, "Expecting DirectiveDef");
						let d = t.type.prototype,
							{
								ngAfterContentInit: c,
								ngAfterContentChecked: f,
								ngAfterViewInit: h,
								ngAfterViewChecked: p,
								ngOnDestroy: m,
							} = d;
						c &&
							(null !== (n = e.contentHooks) && void 0 !== n
								? n
								: (e.contentHooks = [])
							).push(-u, c),
							f &&
								((null !== (r = e.contentHooks) && void 0 !== r
									? r
									: (e.contentHooks = [])
								).push(u, f),
								(null !== (o = e.contentCheckHooks) && void 0 !== o
									? o
									: (e.contentCheckHooks = [])
								).push(u, f)),
							h &&
								(null !== (i = e.viewHooks) && void 0 !== i
									? i
									: (e.viewHooks = [])
								).push(-u, h),
							p &&
								((null !== (s = e.viewHooks) && void 0 !== s
									? s
									: (e.viewHooks = [])
								).push(u, p),
								(null !== (l = e.viewCheckHooks) && void 0 !== l
									? l
									: (e.viewCheckHooks = [])
								).push(u, p)),
							null != m &&
								(null !== (a = e.destroyHooks) && void 0 !== a
									? a
									: (e.destroyHooks = [])
								).push(u, m);
					}
				}
				function nN(e, t, n) {
					nV(e, t, 3, n);
				}
				function nL(e, t, n, r) {
					ngDevMode &&
						B(n, 3, "Init pre-order hooks should not be called more than once"),
						(3 & e[2]) === n && nV(e, t, n, r);
				}
				function n$(e, t) {
					ngDevMode &&
						B(
							t,
							3,
							"Init hooks phase should not be incremented after all init hooks have been run.",
						);
					let n = e[2];
					(3 & n) === t && ((n &= 4095), (n += 1), (e[2] = n));
				}
				function nV(e, t, n, r) {
					ngDevMode &&
						V(
							nu(),
							!1,
							"Hooks should never be run when in check no changes mode.",
						);
					let o = void 0 !== r ? 65535 & e[17] : 0,
						i = null != r ? r : -1,
						s = t.length - 1,
						l = 0;
					for (let a = o; a < s; a++) {
						let o = t[a + 1];
						if ("number" == typeof o) {
							if (((l = t[a]), null != r && l >= r)) break;
						} else {
							let r = t[a] < 0;
							r && (e[17] += 65536),
								(l < i || -1 == i) &&
									((function (e, t, n, r) {
										let o = n[r] < 0,
											i = n[r + 1],
											s = o ? -n[r] : n[r],
											l = e[s];
										if (o) {
											let n = e[2] >> 12;
											n < e[17] >> 16 &&
												(3 & e[2]) === t &&
												((e[2] += 4096), nB(l, i));
										} else nB(l, i);
									})(e, n, t, a),
									(e[17] = (4294901760 & e[17]) + a + 2)),
								a++;
						}
					}
				}
				function nB(e, t) {
					tH(4, e, t);
					let n = tA(null);
					try {
						t.call(e);
					} finally {
						tA(n), tH(5, e, t);
					}
				}
				class nU {
					constructor(e, t, n) {
						(this.factory = e),
							(this.resolving = !1),
							ngDevMode && Z(e, "Factory not specified"),
							ngDevMode &&
								V(typeof e, "function", "Expected factory function."),
							(this.canSeeViewProviders = t),
							(this.injectImpl = n);
					}
				}
				function nH(e) {
					let t = "";
					return (
						1 & e && (t += "|Text"),
						2 & e && (t += "|Element"),
						4 & e && (t += "|Container"),
						8 & e && (t += "|ElementContainer"),
						16 & e && (t += "|Projection"),
						32 & e && (t += "|IcuContainer"),
						64 & e && (t += "|Placeholder"),
						t.length > 0 ? t.substring(1) : t
					);
				}
				function nz(e, t, n) {
					Z(e, "should be called with a TNode"),
						(e.type & t) == 0 &&
							Y(n || `Expected [${nH(t)}] but got ${nH(e.type)}.`);
				}
				function nW(e) {
					return -1 !== e;
				}
				function nq(e) {
					ngDevMode && N(e, "Number expected"),
						ngDevMode && B(e, -1, "Not a valid state.");
					let t = 32767 & e;
					return (
						ngDevMode &&
							q(t, 25, "Parent injector must be pointing past HEADER_OFFSET."),
						32767 & e
					);
				}
				function nG(e, t) {
					let n = e >> 16,
						r = t;
					for (; n > 0; ) (r = r[14]), n--;
					return r;
				}
				let nZ = !0;
				function nY(e) {
					let t = nZ;
					return (nZ = e), t;
				}
				let nQ = 255,
					nK = 0,
					nJ = {};
				function nX(e, t) {
					let n = n1(e, t);
					if (-1 !== n) return n;
					let r = t[1];
					r.firstCreatePass &&
						((e.injectorIndex = t.length),
						n0(r.data, e),
						n0(t, null),
						n0(r.blueprint, null));
					let o = n5(e, t),
						i = e.injectorIndex;
					if (-1 !== o) {
						let e = nq(o),
							n = nG(o, t),
							r = n[1].data;
						for (let o = 0; o < 8; o++) t[i + o] = n[e + o] | r[e + o];
					}
					return (t[i + 8] = o), i;
				}
				function n0(e, t) {
					e.push(0, 0, 0, 0, 0, 0, 0, 0, t);
				}
				function n1(e, t) {
					return -1 === e.injectorIndex ||
						(e.parent && e.parent.injectorIndex === e.injectorIndex) ||
						null === t[e.injectorIndex + 8]
						? -1
						: (ngDevMode && K(t, e.injectorIndex), e.injectorIndex);
				}
				function n5(e, t) {
					if (e.parent && -1 !== e.parent.injectorIndex)
						return e.parent.injectorIndex;
					let n = 0,
						r = null,
						o = t;
					for (; null !== o && null !== (r = rs(o)); ) {
						if (
							(ngDevMode && r && tc(r, o[14]),
							n++,
							(o = o[14]),
							-1 !== r.injectorIndex)
						)
							return r.injectorIndex | (n << 16);
					}
					return -1;
				}
				function n2(e, t, n) {
					!(function (e, t, n) {
						let r;
						ngDevMode &&
							V(t.firstCreatePass, !0, "expected firstCreatePass to be true"),
							"string" == typeof n
								? (r = n.charCodeAt(0) || 0)
								: n.hasOwnProperty(eR) && (r = n[eR]),
							null == r && (r = n[eR] = nK++);
						let o = r & nQ;
						t.data[e + (o >> 5)] |= 1 << o;
					})(e, t, n);
				}
				function n3(e, t, n) {
					if (n & ea.Optional || void 0 !== e) return e;
					R(t, "NodeInjector");
				}
				function n4(e, t, n, r) {
					if (
						(n & ea.Optional && void 0 === r && (r = null),
						(n & (ea.Self | ea.Host)) == 0)
					) {
						let o = e[9],
							i = eu(void 0);
						try {
							if (o) return o.get(t, r, n & ea.Optional);
							return ed(t, r, n & ea.Optional);
						} finally {
							eu(i);
						}
					}
					return n3(r, t, n);
				}
				function n6(e, t, n, r = ea.Default, o) {
					if (null !== e) {
						if (2048 & t[2]) {
							let o = (function (e, t, n, r, o) {
								let i = e,
									s = t;
								for (
									;
									null !== i && null !== s && 2048 & s[2] && !(512 & s[2]);
								) {
									ngDevMode && tc(i, s);
									let e = n8(i, s, n, r | ea.Self, nJ);
									if (e !== nJ) return e;
									let t = i.parent;
									if (!t) {
										let e = s[20];
										if (e) {
											let t = e.get(n, nJ, r);
											if (t !== nJ) return t;
										}
										(t = rs(s)), (s = s[14]);
									}
									i = t;
								}
								return o;
							})(e, t, n, r, nJ);
							if (o !== nJ) return o;
						}
						let o = n8(e, t, n, r, nJ);
						if (o !== nJ) return o;
					}
					return n4(t, n, r, o);
				}
				function n8(e, t, n, r, o) {
					let i = (function (e) {
						if (
							(ngDevMode && Z(e, "token must be defined"), "string" == typeof e)
						)
							return e.charCodeAt(0) || 0;
						let t = e.hasOwnProperty(eR) ? e[eR] : void 0;
						return "number" != typeof t
							? t
							: t >= 0
							? t & nQ
							: (ngDevMode && V(t, -1, "Expecting to get Special Injector Id"),
							  rr);
					})(n);
					if ("function" == typeof i) {
						if (!nj(t, e, r)) return r & ea.Host ? n3(o, n, r) : n4(t, n, r, o);
						try {
							let e = i(r);
							if (null != e || r & ea.Optional) return e;
							R(n);
						} finally {
							nM();
						}
					} else if ("number" == typeof i) {
						let o = null,
							s = n1(e, t),
							l = -1,
							a = r & ea.Host ? t[15][6] : null;
						for (
							(-1 === s || r & ea.SkipSelf) &&
							(-1 !== (l = -1 === s ? n5(e, t) : t[s + 8]) && rt(r, !1)
								? ((o = t[1]), (s = nq(l)), (t = nG(l, t)))
								: (s = -1));
							-1 !== s;
						) {
							ngDevMode && tw(t, s);
							let e = t[1];
							if ((ngDevMode && tc(e.data[s + 8], t), re(i, s, e.data))) {
								let e = (function (e, t, n, r, o, i) {
									let s = t[1],
										l = s.data[e + 8],
										a = null == r ? ta(l) && nZ : r != s && (3 & l.type) != 0,
										u = o & ea.Host && i === l,
										d = n7(l, s, n, a, u);
									return null !== d ? n9(t, s, d, l) : nJ;
								})(s, t, n, o, r, a);
								if (e !== nJ) return e;
							}
							-1 !== (l = t[s + 8]) &&
							rt(r, t[1].data[s + 8] === a) &&
							re(i, s, t)
								? ((o = e), (s = nq(l)), (t = nG(l, t)))
								: (s = -1);
						}
					}
					return o;
				}
				function n7(e, t, n, r, o) {
					let i = e.providerIndexes,
						s = t.data,
						l = 1048575 & i,
						a = e.directiveStart,
						u = e.directiveEnd,
						d = i >> 20,
						c = r ? l : l + d,
						f = o ? l + d : u;
					for (let e = c; e < f; e++) {
						let t = s[e];
						if ((e < a && n === t) || (e >= a && t.type === n)) return e;
					}
					if (o) {
						let e = s[a];
						if (e && td(e) && e.type === n) return a;
					}
					return null;
				}
				function n9(e, t, n, r) {
					let o = e[n],
						i = t.data;
					if (o instanceof nU) {
						let l = o;
						l.resolving && T(P(i[n]));
						let a = nY(l.canSeeViewProviders);
						l.resolving = !0;
						let u = l.injectImpl ? eu(l.injectImpl) : null,
							d = nj(e, r, ea.Default);
						ngDevMode &&
							V(
								d,
								!0,
								"Because flags do not contain `SkipSelf' we expect this to always succeed.",
							);
						try {
							if (
								((o = e[n] = l.factory(void 0, i, e, r)),
								t.firstCreatePass && n >= r.directiveStart)
							) {
								var s;
								ngDevMode &&
									((s = i[n]),
									(void 0 === s.type ||
										void 0 == s.selectors ||
										void 0 === s.inputs) &&
										Y(
											"Expected a DirectiveDef/ComponentDef and this object does not seem to have the expected shape.",
										)),
									!(function (e, t, n) {
										var r, o, i, s, l;
										ngDevMode && tb(n);
										let {
											ngOnChanges: a,
											ngOnInit: u,
											ngDoCheck: d,
										} = t.type.prototype;
										if (a) {
											let i = tR(t);
											(null !== (r = n.preOrderHooks) && void 0 !== r
												? r
												: (n.preOrderHooks = [])
											).push(e, i),
												(null !== (o = n.preOrderCheckHooks) && void 0 !== o
													? o
													: (n.preOrderCheckHooks = [])
												).push(e, i);
										}
										u &&
											(null !== (i = n.preOrderHooks) && void 0 !== i
												? i
												: (n.preOrderHooks = [])
											).push(0 - e, u),
											d &&
												((null !== (s = n.preOrderHooks) && void 0 !== s
													? s
													: (n.preOrderHooks = [])
												).push(e, d),
												(null !== (l = n.preOrderCheckHooks) && void 0 !== l
													? l
													: (n.preOrderCheckHooks = [])
												).push(e, d));
									})(n, i[n], t);
							}
						} finally {
							null !== u && eu(u), nY(a), (l.resolving = !1), nM();
						}
					}
					return o;
				}
				function re(e, t, n) {
					let r = n[t + (e >> 5)];
					return !!(r & (1 << e));
				}
				function rt(e, t) {
					return !(e & ea.Self) && !(e & ea.Host && t);
				}
				class rn {
					get(e, t, n) {
						return n6(this._tNode, this._lView, e, eD(n), t);
					}
					constructor(e, t) {
						(this._tNode = e), (this._lView = t);
					}
				}
				function rr() {
					return new rn(nr(), t9());
				}
				function ro(e) {
					return eC(() => {
						let t = e.prototype.constructor,
							n = t[eF] || ri(t),
							r = Object.prototype,
							o = Object.getPrototypeOf(e.prototype).constructor;
						for (; o && o !== r; ) {
							let e = o[eF] || ri(o);
							if (e && e !== n) return e;
							o = Object.getPrototypeOf(o);
						}
						return (e) => new e();
					});
				}
				function ri(e) {
					return C(e)
						? () => {
								let t = ri(M(e));
								return t && t();
						  }
						: tM(e);
				}
				function rs(e) {
					let t = e[1],
						n = t.type;
					return 2 === n
						? (ngDevMode &&
								Z(
									t.declTNode,
									"Embedded TNodes should have declaration parents.",
								),
						  t.declTNode)
						: 1 === n
						? e[6]
						: null;
				}
				function rl(e) {
					return (function (e, t) {
						if (
							(ngDevMode && nz(e, 15),
							ngDevMode && Z(e, "expecting tNode"),
							"class" === t)
						)
							return e.classes;
						if ("style" === t) return e.styles;
						let n = e.attrs;
						if (n) {
							let e = n.length,
								r = 0;
							for (; r < e; ) {
								let o = n[r];
								if (eV(o)) break;
								if (0 === o) r += 2;
								else if ("number" == typeof o)
									for (r++; r < e && "string" == typeof n[r]; ) r++;
								else {
									if (o === t) return n[r + 1];
									r += 2;
								}
							}
						}
						return null;
					})(nr(), e);
				}
				let ra = "__annotations__",
					ru = "__parameters__",
					rd = "__prop__metadata__";
				function rc(e, t, n, r, o) {
					return eC(() => {
						let i = rf(t);
						function s(...e) {
							if (this instanceof s) return i.call(this, ...e), this;
							let t = new s(...e);
							return function (n) {
								o && o(n, ...e);
								let i = n.hasOwnProperty(ra)
									? n[ra]
									: Object.defineProperty(n, ra, { value: [] })[ra];
								return i.push(t), r && r(n), n;
							};
						}
						return (
							n && (s.prototype = Object.create(n.prototype)),
							(s.prototype.ngMetadataName = e),
							(s.annotationCls = s),
							s
						);
					});
				}
				function rf(e) {
					return function (...t) {
						if (e) {
							let n = e(...t);
							for (let e in n) this[e] = n[e];
						}
					};
				}
				function rh(e, t, n) {
					return eC(() => {
						let r = rf(t);
						function o(...e) {
							if (this instanceof o) return r.apply(this, e), this;
							let t = new o(...e);
							return (n.annotation = t), n;
							function n(e, n, r) {
								let o = e.hasOwnProperty(ru)
									? e[ru]
									: Object.defineProperty(e, ru, { value: [] })[ru];
								for (; o.length <= r; ) o.push(null);
								return (o[r] = o[r] || []).push(t), e;
							}
						}
						return (
							n && (o.prototype = Object.create(n.prototype)),
							(o.prototype.ngMetadataName = e),
							(o.annotationCls = o),
							o
						);
					});
				}
				function rp(e, t, n, r) {
					return eC(() => {
						let o = rf(t);
						function i(...e) {
							if (this instanceof i) return o.apply(this, e), this;
							let t = new i(...e);
							return function (n, o) {
								if (void 0 === n)
									throw Error(
										"Standard Angular field decorators are not supported in JIT mode.",
									);
								let i = n.constructor,
									s = i.hasOwnProperty(rd)
										? i[rd]
										: Object.defineProperty(i, rd, { value: {} })[rd];
								(s[o] = (s.hasOwnProperty(o) && s[o]) || []),
									s[o].unshift(t),
									r && r(n, o, ...e);
							};
						}
						return (
							n && (i.prototype = Object.create(n.prototype)),
							(i.prototype.ngMetadataName = e),
							(i.annotationCls = i),
							i
						);
					});
				}
				let rm = rh("Attribute", (e) => ({
					attributeName: e,
					__NG_ELEMENT_ID__: () => rl(e),
				}));
				class rg {}
				rp(
					"ContentChildren",
					(e, t = {}) =>
						m._(
							{
								selector: e,
								first: !1,
								isViewQuery: !1,
								descendants: !1,
								emitDistinctChangesOnly: !0,
							},
							t,
						),
					rg,
				),
					rp(
						"ContentChild",
						(e, t = {}) =>
							m._(
								{ selector: e, first: !0, isViewQuery: !1, descendants: !0 },
								t,
							),
						rg,
					),
					rp(
						"ViewChildren",
						(e, t = {}) =>
							m._(
								{
									selector: e,
									first: !1,
									isViewQuery: !0,
									descendants: !0,
									emitDistinctChangesOnly: !0,
								},
								t,
							),
						rg,
					),
					rp(
						"ViewChild",
						(e, t) =>
							m._(
								{ selector: e, first: !0, isViewQuery: !0, descendants: !0 },
								t,
							),
						rg,
					);
				var rv =
						(((rv = rv || {})[(rv.Directive = 0)] = "Directive"),
						(rv[(rv.Component = 1)] = "Component"),
						(rv[(rv.Injectable = 2)] = "Injectable"),
						(rv[(rv.Pipe = 3)] = "Pipe"),
						(rv[(rv.NgModule = 4)] = "NgModule"),
						rv),
					ry =
						(((ry = ry || {})[(ry.Directive = 0)] = "Directive"),
						(ry[(ry.Pipe = 1)] = "Pipe"),
						(ry[(ry.NgModule = 2)] = "NgModule"),
						ry),
					rb =
						(((rb = rb || {})[(rb.Emulated = 0)] = "Emulated"),
						(rb[(rb.None = 2)] = "None"),
						(rb[(rb.ShadowDom = 3)] = "ShadowDom"),
						rb);
				function r_(e) {
					let t = ec.ng;
					if (t && t.ɵcompilerFacade) return t.ɵcompilerFacade;
					if ("undefined" == typeof ngDevMode || ngDevMode) {
						console.error(`JIT compilation failed for ${e.kind}`, e.type);
						let t = `The ${e.kind} '${e.type.name}' needs to be compiled using the JIT compiler, but '@angular/compiler' is not available.

`;
						throw (
							(1 === e.usage
								? (t += `The ${e.kind} is part of a library that has been partially compiled.
However, the Angular Linker has not processed the library such that JIT compilation is used as fallback.

Ideally, the library is processed using the Angular Linker to become fully AOT compiled.
`)
								: (t += `JIT compilation is discouraged for production use-cases! Consider using AOT mode instead.
`),
							Error(
								(t += `Alternatively, the JIT compiler should be loaded by bootstrapping using '@angular/platform-browser-dynamic' or '@angular/platform-server',
or manually provide the compiler with 'import "@angular/compiler";' before bootstrapping.`),
							))
						);
					}
					throw Error("JIT compiler unavailable");
				}
				let rj = Function;
				function rx(e) {
					return "function" == typeof e;
				}
				function rD(e) {
					return e.flat(Number.POSITIVE_INFINITY);
				}
				function rw(e, t) {
					e.forEach((e) => (Array.isArray(e) ? rw(e, t) : t(e)));
				}
				function rM(e, t, n) {
					t >= e.length ? e.push(n) : e.splice(t, 0, n);
				}
				function rC(e, t) {
					return t >= e.length - 1 ? e.pop() : e.splice(t, 1)[0];
				}
				function rS(e, t) {
					let n = [];
					for (let r = 0; r < e; r++) n.push(t);
					return n;
				}
				function rE(e, t, n) {
					let r = (function (e, t) {
						return rI(e, t, 1);
					})(e, t);
					return (
						r >= 0
							? (e[1 | r] = n)
							: !(function (e, t, n, r) {
									ngDevMode && W(t, e.length, "Can't insert past array end.");
									let o = e.length;
									if (o == t) e.push(n, r);
									else if (1 === o) e.push(r, e[0]), (e[0] = n);
									else {
										for (o--, e.push(e[o - 1], e[o]); o > t; ) {
											let t = o - 2;
											(e[o] = e[t]), o--;
										}
										(e[t] = n), (e[t + 1] = r);
									}
							  })(e, (r = ~r), t, n),
						r
					);
				}
				function rO(e, t) {
					let n = (function (e, t) {
						return rI(e, t, 1);
					})(e, t);
					if (n >= 0) return e[1 | n];
				}
				function rA(e, t) {
					return rI(e, t, 1);
				}
				function rI(e, t, n) {
					ngDevMode && V(Array.isArray(e), !0, "Expecting an array");
					let r = 0,
						o = e.length >> n;
					for (; o !== r; ) {
						let i = r + ((o - r) >> 1),
							s = e[i << n];
						if (t === s) return i << n;
						s > t ? (o = i) : (r = i + 1);
					}
					return ~(o << n);
				}
				let rP =
						/^function\s+\S+\(\)\s*{[\s\S]+\.apply\(this,\s*(arguments|(?:[^()]+\(\[\],)?[^()]+\(arguments\).*)\)/,
					rT = /^class\s+[A-Za-z\d$_]*\s*extends\s+[^{]+{/,
					rk =
						/^class\s+[A-Za-z\d$_]*\s*extends\s+[^{]+{[\s\S]*constructor\s*\(/,
					rF =
						/^class\s+[A-Za-z\d$_]*\s*extends\s+[^{]+{[\s\S]*constructor\s*\(\)\s*{[^}]*super\(\.\.\.arguments\)/;
				class rR {
					factory(e) {
						return (...t) => new e(...t);
					}
					_zipTypesAndAnnotations(e, t) {
						let n;
						n = void 0 === e ? rS(t.length) : rS(e.length);
						for (let r = 0; r < n.length; r++)
							void 0 === e
								? (n[r] = [])
								: e[r] && e[r] != Object
								? (n[r] = [e[r]])
								: (n[r] = []),
								t && null != t[r] && (n[r] = n[r].concat(t[r]));
						return n;
					}
					_ownParameters(e, t) {
						var n;
						let r = e.toString();
						if (
							((n = r), rP.test(n) || rF.test(n) || (rT.test(n) && !rk.test(n)))
						)
							return null;
						if (e.parameters && e.parameters !== t.parameters)
							return e.parameters;
						let o = e.ctorParameters;
						if (o && o !== t.ctorParameters) {
							let e = "function" == typeof o ? o() : o,
								t = e.map((e) => e && e.type),
								n = e.map((e) => e && rN(e.decorators));
							return this._zipTypesAndAnnotations(t, n);
						}
						let i = e.hasOwnProperty(ru) && e[ru],
							s =
								this._reflect &&
								this._reflect.getOwnMetadata &&
								this._reflect.getOwnMetadata("design:paramtypes", e);
						return s || i ? this._zipTypesAndAnnotations(s, i) : rS(e.length);
					}
					parameters(e) {
						if (!rx(e)) return [];
						let t = rL(e),
							n = this._ownParameters(e, t);
						return !n && t !== Object && (n = this.parameters(t)), n || [];
					}
					_ownAnnotations(e, t) {
						if (e.annotations && e.annotations !== t.annotations) {
							let t = e.annotations;
							return (
								"function" == typeof t && t.annotations && (t = t.annotations),
								t
							);
						}
						return e.decorators && e.decorators !== t.decorators
							? rN(e.decorators)
							: e.hasOwnProperty(ra)
							? e[ra]
							: null;
					}
					annotations(e) {
						if (!rx(e)) return [];
						let t = rL(e),
							n = this._ownAnnotations(e, t) || [],
							r = t !== Object ? this.annotations(t) : [];
						return r.concat(n);
					}
					_ownPropMetadata(e, t) {
						if (e.propMetadata && e.propMetadata !== t.propMetadata) {
							let t = e.propMetadata;
							return (
								"function" == typeof t &&
									t.propMetadata &&
									(t = t.propMetadata),
								t
							);
						}
						if (e.propDecorators && e.propDecorators !== t.propDecorators) {
							let t = e.propDecorators,
								n = {};
							return (
								Object.keys(t).forEach((e) => {
									n[e] = rN(t[e]);
								}),
								n
							);
						}
						return e.hasOwnProperty(rd) ? e[rd] : null;
					}
					propMetadata(e) {
						if (!rx(e)) return {};
						let t = rL(e),
							n = {};
						if (t !== Object) {
							let e = this.propMetadata(t);
							Object.keys(e).forEach((t) => {
								n[t] = e[t];
							});
						}
						let r = this._ownPropMetadata(e, t);
						return (
							r &&
								Object.keys(r).forEach((e) => {
									let t = [];
									n.hasOwnProperty(e) && t.push(...n[e]),
										t.push(...r[e]),
										(n[e] = t);
								}),
							n
						);
					}
					ownPropMetadata(e) {
						return (rx(e) && this._ownPropMetadata(e, rL(e))) || {};
					}
					hasLifecycleHook(e, t) {
						return e instanceof rj && t in e.prototype;
					}
					constructor(e) {
						this._reflect = e || ec.Reflect;
					}
				}
				function rN(e) {
					return e
						? e.map((e) => {
								let t = e.type,
									n = t.annotationCls,
									r = e.args ? e.args : [];
								return new n(...r);
						  })
						: [];
				}
				function rL(e) {
					let t = e.prototype ? Object.getPrototypeOf(e.prototype) : null,
						n = t ? t.constructor : null;
					return n || Object;
				}
				let r$ = eM(
						rh("Inject", (e) => ({ token: e })),
						-1,
					),
					rV = eM(rh("Optional"), 8),
					rB = eM(rh("Self"), 2),
					rU = eM(rh("SkipSelf"), 4),
					rH = eM(rh("Host"), 1),
					rz = null;
				function rW() {
					return (rz = rz || new rR());
				}
				function rq(e) {
					return rG(rW().parameters(e));
				}
				function rG(e) {
					return e.map((e) =>
						(function (e) {
							let t = {
								token: null,
								attribute: null,
								host: !1,
								optional: !1,
								self: !1,
								skipSelf: !1,
							};
							if (Array.isArray(e) && e.length > 0)
								for (let n = 0; n < e.length; n++) {
									let r = e[n];
									if (void 0 === r) continue;
									let o = Object.getPrototypeOf(r);
									if (r instanceof rV || "Optional" === o.ngMetadataName)
										t.optional = !0;
									else if (r instanceof rU || "SkipSelf" === o.ngMetadataName)
										t.skipSelf = !0;
									else if (r instanceof rB || "Self" === o.ngMetadataName)
										t.self = !0;
									else if (r instanceof rH || "Host" === o.ngMetadataName)
										t.host = !0;
									else if (r instanceof r$) t.token = r.token;
									else if (r instanceof rm) {
										if (void 0 === r.attributeName)
											throw new O(
												204,
												ngDevMode && "Attribute name must be defined.",
											);
										t.attribute = r.attributeName;
									} else t.token = r;
								}
							else
								void 0 === e || (Array.isArray(e) && 0 === e.length)
									? (t.token = null)
									: (t.token = e);
							return t;
						})(e),
					);
				}
				let rZ = new Map(),
					rY = new Set();
				function rQ(e) {
					return !!(
						(e.templateUrl && !e.hasOwnProperty("template")) ||
						(e.styleUrls && e.styleUrls.length)
					);
				}
				function rK(e) {
					return "string" == typeof e ? e : e.text();
				}
				let rJ = new Map();
				function rX(e, t) {
					let n = rJ.get(t) || null;
					!(function (e, t, n) {
						if (t && t !== n)
							throw Error(
								`Duplicate module registered for ${e} - ${j(t)} vs ${j(
									t.name,
								)}`,
							);
					})(t, n, e),
						rJ.set(t, e);
				}
				let r0 = { name: "custom-elements" },
					r1 = { name: "no-errors-schema" };
				function r5(e, t, n, r) {
					!t && 4 === n && (t = "ng-template");
					let o = r4(r),
						i = r6(r),
						s = `Can't bind to '${e}' since it isn't a known property of '${t}'${i}.`,
						l = `'${o ? "@Component" : "@NgModule"}.schemas'`,
						a = o
							? "included in the '@Component.imports' of this component"
							: "a part of an @NgModule where this component is declared";
					if (r8.has(e)) {
						let t = r8.get(e);
						s += `
If the '${e}' is an Angular control flow directive, please make sure that either the '${t}' directive or the 'CommonModule' is ${a}.`;
					} else
						(s += `
1. If '${t}' is an Angular component and it has the '${e}' input, then verify that it is ${a}.`),
							t && t.indexOf("-") > -1
								? (s += `
2. If '${t}' is a Web Component then add 'CUSTOM_ELEMENTS_SCHEMA' to the ${l} of this component to suppress this message.
3. To allow any property add 'NO_ERRORS_SCHEMA' to the ${l} of this component.`)
								: (s += `
2. To allow any property add 'NO_ERRORS_SCHEMA' to the ${l} of this component.`);
					r2(s);
				}
				function r2(e) {
					console.error(A(303, e));
				}
				function r3(e) {
					ngDevMode || Y("Must never be called in production mode");
					let t = e[15],
						n = t[8];
					return n ? (n.constructor ? e6(n.constructor) : null) : null;
				}
				function r4(e) {
					ngDevMode || Y("Must never be called in production mode");
					let t = r3(e);
					return !!(null == t ? void 0 : t.standalone);
				}
				function r6(e) {
					var t;
					ngDevMode || Y("Must never be called in production mode");
					let n = r3(e),
						r =
							null == n
								? void 0
								: null === (t = n.type) || void 0 === t
								? void 0
								: t.name;
					return r ? ` (used in the '${r}' component template)` : "";
				}
				let r8 = new Map([
					["ngIf", "NgIf"],
					["ngFor", "NgFor"],
					["ngSwitchCase", "NgSwitchCase"],
					["ngSwitchDefault", "NgSwitchDefault"],
				]);
				function r7(e, t) {
					if (null !== e)
						for (let n = 0; n < e.length; n++) {
							let r = e[n];
							if (r === r1 || (r === r0 && t && t.indexOf("-") > -1)) return !0;
						}
					return !1;
				}
				var r9 =
					(((r9 = r9 || {})[(r9.Important = 1)] = "Important"),
					(r9[(r9.DashCase = 2)] = "DashCase"),
					r9);
				let oe = /^>|^->|<!--|-->|--!>|<!-$/g,
					ot = /(<|>)/;
				function on(e) {
					return e.replace(oe, (e) => e.replace(ot, "​$1​"));
				}
				let or = new Map(),
					oo = 0;
				function oi(e) {
					return (
						ngDevMode && N(e, "ID used for LView lookup must be a number"),
						or.get(e) || null
					);
				}
				class os {
					get lView() {
						return oi(this.lViewId);
					}
					constructor(e, t, n) {
						(this.lViewId = e), (this.nodeIndex = t), (this.native = n);
					}
				}
				function ol(e) {
					let t = of(e);
					if (t) {
						if (ti(t)) {
							let n, r, o;
							let i = t;
							if (op(e)) {
								if (-1 == (n = og(i, e)))
									throw Error(
										"The provided component was not found in the application",
									);
								r = e;
							} else if (
								(function (e) {
									return e && e.constructor && e.constructor.ɵdir;
								})(e)
							) {
								if (
									-1 ==
									(n = (function (e, t) {
										let n = e[1].firstChild;
										for (; n; ) {
											let r = n.directiveStart,
												o = n.directiveEnd;
											for (let i = r; i < o; i++)
												if (e[i] === t) return n.index;
											n = (function (e) {
												if (e.child) return e.child;
												if (e.next) return e.next;
												for (; e.parent && !e.parent.next; ) e = e.parent;
												return e.parent && e.parent.next;
											})(n);
										}
										return -1;
									})(i, e))
								)
									throw Error(
										"The provided directive was not found in the application",
									);
								o = ov(n, i);
							} else if (-1 == (n = om(i, e))) return null;
							let s = tW(i[n]),
								l = of(s),
								a = l && !Array.isArray(l) ? l : oa(i, n, s);
							if (
								(r &&
									void 0 === a.component &&
									((a.component = r), oc(a.component, a)),
								o && void 0 === a.directives)
							) {
								a.directives = o;
								for (let e = 0; e < o.length; e++) oc(o[e], a);
							}
							oc(a.native, a), (t = a);
						}
					} else {
						ngDevMode && Q(e);
						let n = e;
						for (; (n = n.parentNode); ) {
							let r = of(n);
							if (r) {
								let n = Array.isArray(r) ? r : r.lView;
								if (!n) return null;
								let o = om(n, e);
								if (o >= 0) {
									let e = tW(n[o]),
										r = oa(n, o, e);
									oc(e, r), (t = r);
									break;
								}
							}
						}
					}
					return t || null;
				}
				function oa(e, t, n) {
					return new os(e[19], t, n);
				}
				function ou(e) {
					let t,
						n = of(e);
					if (ti(n)) {
						let r = og(n, e);
						t = tQ(r, n);
						let o = oa(n, r, t[0]);
						(o.component = e), oc(e, o), oc(o.native, o);
					} else {
						let e = n.lView;
						ngDevMode && ty(e), (t = tQ(n.nodeIndex, e));
					}
					return t;
				}
				let od = "__ngContext__";
				function oc(e, t) {
					if ((ngDevMode && Z(e, "Target expected"), ti(t))) {
						var n;
						(e[od] = t[19]),
							(n = t),
							ngDevMode &&
								N(n[19], "LView must have an ID in order to be registered"),
							or.set(n[19], n);
					} else e[od] = t;
				}
				function of(e) {
					ngDevMode && Z(e, "Target expected");
					let t = e[od];
					return "number" == typeof t ? oi(t) : t || null;
				}
				function oh(e) {
					let t = of(e);
					return t ? (ti(t) ? t : t.lView) : null;
				}
				function op(e) {
					return e && e.constructor && e.constructor.ɵcmp;
				}
				function om(e, t) {
					let n = e[1];
					for (let r = 25; r < n.bindingStartIndex; r++)
						if (tW(e[r]) === t) return r;
					return -1;
				}
				function og(e, t) {
					let n = e[1].components;
					if (n)
						for (let r = 0; r < n.length; r++) {
							let o = n[r],
								i = tQ(o, e);
							if (i[8] === t) return o;
						}
					else {
						let n = tQ(25, e),
							r = n[8];
						if (r === t) return 25;
					}
					return -1;
				}
				function ov(e, t) {
					let n = t[1].data[e];
					if (0 === n.directiveStart) return eA;
					let r = [];
					for (let e = n.directiveStart; e < n.directiveEnd; e++) {
						let n = t[e];
						!op(n) && r.push(n);
					}
					return r;
				}
				function oy(e, t) {
					return o(e, t);
				}
				function ob(e) {
					ngDevMode && ty(e);
					let t = e[3];
					return ts(t) ? t[3] : t;
				}
				function o_(e) {
					return ox(e[12]);
				}
				function oj(e) {
					return ox(e[4]);
				}
				function ox(e) {
					for (; null !== e && !ts(e); ) e = e[4];
					return e;
				}
				function oD(e, t, n, r, o) {
					if (null != r) {
						let i;
						let s = !1;
						ts(r)
							? (i = r)
							: ti(r) &&
							  ((s = !0),
							  ngDevMode &&
									Z(r[0], "HOST must be defined for a component LView"),
							  (r = r[0]));
						let l = tW(r);
						0 === e && null !== n
							? null == o
								? oF(t, n, l)
								: ok(t, n, l, o || null, !0)
							: 1 === e && null !== n
							? ok(t, n, l, o || null, !0)
							: 2 === e
							? oW(t, l, s)
							: 3 === e &&
							  (ngDevMode && ngDevMode.rendererDestroyNode++,
							  t.destroyNode(l)),
							null != i &&
								(function (e, t, n, r, o) {
									ngDevMode && tg(n);
									let i = n[7],
										s = tW(n);
									i !== s && oD(t, e, r, i, o);
									for (let o = 11; o < n.length; o++) {
										let s = n[o];
										oG(s[1], s, e, t, r, i);
									}
								})(t, e, i, n, o);
					}
				}
				function ow(e, t) {
					return (
						ngDevMode && ngDevMode.rendererCreateTextNode++,
						ngDevMode && ngDevMode.rendererSetText++,
						e.createText(t)
					);
				}
				function oM(e, t, n) {
					ngDevMode && ngDevMode.rendererSetText++, e.setValue(t, n);
				}
				function oC(e, t) {
					return (
						ngDevMode && ngDevMode.rendererCreateComment++,
						e.createComment(on(t))
					);
				}
				function oS(e, t, n) {
					return (
						ngDevMode && ngDevMode.rendererCreateElement++,
						e.createElement(t, n)
					);
				}
				function oE(e, t) {
					ngDevMode && tg(e),
						ngDevMode &&
							Z(
								e[9],
								"A projected view should belong to a non-empty projected views collection",
							);
					let n = e[9],
						r = n.indexOf(t),
						o = t[3];
					ngDevMode && tg(o), t1(t), n.splice(r, 1);
				}
				function oO(e, t) {
					if (e.length <= 11) return;
					let n = 11 + t,
						r = e[n];
					if (r) {
						let o = r[16];
						null !== o && o !== e && oE(o, r), t > 0 && (e[n - 1][4] = r[4]);
						let i = rC(e, 11 + t);
						!(function (e, t) {
							let n = t[11];
							oG(e, t, n, 2, null, null), (t[0] = null), (t[6] = null);
						})(r[1], r);
						let s = i[18];
						null !== s && s.detachView(i[1]),
							(r[3] = null),
							(r[4] = null),
							(r[2] &= -129);
					}
					return r;
				}
				function oA(e, t) {
					if (!(256 & t[2])) {
						var n, r;
						let o = t[11];
						null === (n = t[23]) || void 0 === n || n.destroy(),
							null === (r = t[24]) || void 0 === r || r.destroy(),
							o.destroyNode && oG(e, t, o, 3, null, null),
							!(function (e) {
								let t = e[12];
								if (!t) return oI(e[1], e);
								for (; t; ) {
									let n = null;
									if (ti(t)) n = t[12];
									else {
										ngDevMode && tg(t);
										let e = t[11];
										e && (n = e);
									}
									if (!n) {
										for (; t && !t[4] && t !== e; )
											ti(t) && oI(t[1], t), (t = t[3]);
										null === t && (t = e),
											ti(t) && oI(t[1], t),
											(n = t && t[4]);
									}
									t = n;
								}
							})(t);
					}
				}
				function oI(e, t) {
					if (!(256 & t[2])) {
						var n;
						(t[2] &= -129),
							(t[2] |= 256),
							(function (e, t) {
								let n;
								if (null != e && null != (n = e.destroyHooks))
									for (let e = 0; e < n.length; e += 2) {
										let r = t[n[e]];
										if (!(r instanceof nU)) {
											let t = n[e + 1];
											if (Array.isArray(t))
												for (let e = 0; e < t.length; e += 2) {
													let n = r[t[e]],
														o = t[e + 1];
													tH(4, n, o);
													try {
														o.call(n);
													} finally {
														tH(5, n, o);
													}
												}
											else {
												tH(4, r, t);
												try {
													t.call(r);
												} finally {
													tH(5, r, t);
												}
											}
										}
									}
							})(e, t),
							(function (e, t) {
								let n = e.cleanup,
									r = t[7];
								if (null !== n)
									for (let e = 0; e < n.length - 1; e += 2)
										if ("string" == typeof n[e]) {
											let t = n[e + 3];
											ngDevMode && N(t, "cleanup target must be a number"),
												t >= 0 ? r[t]() : r[-t].unsubscribe(),
												(e += 2);
										} else {
											let t = r[n[e + 1]];
											n[e].call(t);
										}
								null !== r && (t[7] = null);
								let o = t[21];
								if (null !== o) {
									for (let e = 0; e < o.length; e++) {
										var i, s;
										let t = o[e];
										ngDevMode &&
											((s = "Expecting destroy hook to be a function."),
											"function" != typeof (i = t) &&
												Y(
													s,
													null === i ? "null" : typeof i,
													"function",
													"===",
												)),
											t();
									}
									t[21] = null;
								}
							})(e, t),
							1 === t[1].type &&
								(ngDevMode && ngDevMode.rendererDestroy++, t[11].destroy());
						let r = t[16];
						if (null !== r && ts(t[3])) {
							r !== t[3] && oE(r, t);
							let n = t[18];
							null !== n && n.detachView(e);
						}
						(n = t),
							ngDevMode &&
								N(
									n[19],
									"Cannot stop tracking an LView that does not have an ID",
								),
							or.delete(n[19]);
					}
				}
				function oP(e, t, n) {
					return oT(e, t.parent, n);
				}
				function oT(e, t, n) {
					let r = t;
					for (; null !== r && 40 & r.type; ) r = (t = r).parent;
					if (null === r) return n[0];
					{
						ngDevMode && nz(r, 7);
						let { componentOffset: t } = r;
						if (t > -1) {
							ngDevMode && tc(r, n);
							let { encapsulation: o } = e.data[r.directiveStart + t];
							if (o === eE.None || o === eE.Emulated) return null;
						}
						return tG(r, n);
					}
				}
				function ok(e, t, n, r, o) {
					ngDevMode && ngDevMode.rendererInsertBefore++,
						e.insertBefore(t, n, r, o);
				}
				function oF(e, t, n) {
					ngDevMode && ngDevMode.rendererAppendChild++,
						ngDevMode && Z(t, "parent node must be defined"),
						e.appendChild(t, n);
				}
				function oR(e, t, n, r, o) {
					null !== r ? ok(e, t, n, r, o) : oF(e, t, n);
				}
				function oN(e, t) {
					return e.parentNode(t);
				}
				function oL(e, t, n) {
					return oV(e, t, n);
				}
				function o$(e, t, n) {
					return 40 & e.type ? tG(e, n) : null;
				}
				let oV = o$;
				function oB(e, t) {
					(oV = e), (i = t);
				}
				function oU(e, t, n, r) {
					var o, s, l;
					let a = ((o = e), (s = r), (l = t), oT(o, s.parent, l)),
						u = t[11],
						d = r.parent || t[6],
						c = oV(d, r, t);
					if (null != a) {
						if (Array.isArray(n))
							for (let e = 0; e < n.length; e++) oR(u, a, n[e], c, !1);
						else oR(u, a, n, c, !1);
					}
					void 0 !== i && i(u, r, t, n, a);
				}
				function oH(e, t) {
					if (null !== t) {
						var n, r;
						let o = e[15],
							i = o[6],
							s = t.projection;
						return (
							ngDevMode &&
								(Z((n = e)[15], "Component views should exist."),
								Z(
									n[15][6].projection,
									"Components with projection nodes (<ng-content>) must have projection slots defined.",
								)),
							i.projection[s]
						);
					}
					return null;
				}
				function oz(e, t) {
					let n = 11 + e + 1;
					if (n < t.length) {
						let e = t[n],
							r = e[1].firstChild;
						if (null !== r)
							return (function e(t, n) {
								if (null !== n) {
									ngDevMode && nz(n, 63);
									let r = n.type;
									if (3 & r) return tG(n, t);
									if (4 & r) return oz(-1, t[n.index]);
									else if (8 & r) {
										let r = n.child;
										if (null !== r) return e(t, r);
										{
											let e = t[n.index];
											return ts(e) ? oz(-1, e) : tW(e);
										}
									} else {
										if (32 & r) return o(n, t)() || tW(t[n.index]);
										let i = oH(t, n);
										if (null === i) return e(t, n.next);
										{
											if (Array.isArray(i)) return i[0];
											let n = ob(t[15]);
											return ngDevMode && tD(n), e(n, i);
										}
									}
								}
								return null;
							})(e, r);
					}
					return t[7];
				}
				function oW(e, t, n) {
					ngDevMode && ngDevMode.rendererRemoveNode++;
					let r = oN(e, t);
					r &&
						!(function (e, t, n, r) {
							e.removeChild(t, n, r);
						})(e, r, t, n);
				}
				function oq(e, t, n, r, i, s, l) {
					for (; null != n; ) {
						ngDevMode && tc(n, r), ngDevMode && nz(n, 63);
						let a = r[n.index],
							u = n.type;
						if (
							(l && 0 === t && (a && oc(tW(a), r), (n.flags |= 2)),
							(32 & n.flags) != 32)
						) {
							if (8 & u) oq(e, t, n.child, r, i, s, !1), oD(t, e, i, a, s);
							else if (32 & u) {
								let l;
								let u = o(n, r);
								for (; (l = u()); ) oD(t, e, i, l, s);
								oD(t, e, i, a, s);
							} else
								16 & u
									? oZ(e, t, r, n, i, s)
									: (ngDevMode && nz(n, 7), oD(t, e, i, a, s));
						}
						n = l ? n.projectionNext : n.next;
					}
				}
				function oG(e, t, n, r, o, i) {
					oq(n, r, e.firstChild, t, o, i, !1);
				}
				function oZ(e, t, n, r, o, i) {
					let s = n[15],
						l = s[6];
					ngDevMode &&
						V(typeof r.projection, "number", "expecting projection index");
					let a = l.projection[r.projection];
					if (Array.isArray(a))
						for (let n = 0; n < a.length; n++) {
							let r = a[n];
							oD(t, e, o, r, i);
						}
					else {
						let n = s[3];
						oq(e, t, a, n, o, i, !0);
					}
				}
				function oY(e, t, n) {
					ngDevMode && $(n, "'newValue' should be a string"),
						"" === n
							? e.removeAttribute(t, "class")
							: e.setAttribute(t, "class", n),
						ngDevMode && ngDevMode.rendererSetClassName++;
				}
				function oQ(e, t, n) {
					let { mergedAttrs: r, classes: o, styles: i } = n;
					if (
						(null !== r && e$(e, t, r), null !== o && oY(e, t, o), null !== i)
					) {
						var s, l, a;
						(s = e),
							(l = t),
							(a = i),
							ngDevMode && $(a, "'newValue' should be a string"),
							s.setAttribute(l, "style", a),
							ngDevMode && ngDevMode.rendererSetStyle++;
					}
				}
				function oK() {
					if (void 0 === s && ((s = null), ec.trustedTypes))
						try {
							s = ec.trustedTypes.createPolicy("angular", {
								createHTML: (e) => e,
								createScript: (e) => e,
								createScriptURL: (e) => e,
							});
						} catch (e) {}
					return s;
				}
				function oJ(e) {
					var t;
					return (
						(null === (t = oK()) || void 0 === t ? void 0 : t.createHTML(e)) ||
						e
					);
				}
				function oX(e, t, n) {
					let r = t9(),
						o = nO(),
						i = tG(o, r);
					if (2 === o.type && "iframe" === t.toLowerCase()) {
						(i.src = ""), (i.srcdoc = oJ("")), oW(r[11], i);
						let e =
							ngDevMode &&
							`Angular has detected that the \`${n}\` was applied as a binding to an <iframe>${r6(
								r,
							)}. For security reasons, the \`${n}\` can be set on an <iframe> as a static attribute only. 
To fix this, switch the \`${n}\` binding to a static attribute in a template or in host bindings section.`;
						throw new O(-910, e);
					}
					return e;
				}
				function o0(e) {
					c = e;
				}
				function o1() {
					return void 0 !== c
						? c
						: "undefined" != typeof document
						? document
						: void 0;
				}
				function o5() {
					if (void 0 === l && ((l = null), ec.trustedTypes))
						try {
							l = ec.trustedTypes.createPolicy("angular#unsafe-bypass", {
								createHTML: (e) => e,
								createScript: (e) => e,
								createScriptURL: (e) => e,
							});
						} catch (e) {}
					return l;
				}
				function o2(e) {
					var t;
					return (
						(null === (t = o5()) || void 0 === t ? void 0 : t.createHTML(e)) ||
						e
					);
				}
				function o3(e) {
					var t;
					return (
						(null === (t = o5()) || void 0 === t
							? void 0
							: t.createScript(e)) || e
					);
				}
				function o4(e) {
					var t;
					return (
						(null === (t = o5()) || void 0 === t
							? void 0
							: t.createScriptURL(e)) || e
					);
				}
				class o6 {
					toString() {
						return `SafeValue must use [property]=binding: ${this.changingThisBreaksApplicationSecurity} (see ${E})`;
					}
					constructor(e) {
						this.changingThisBreaksApplicationSecurity = e;
					}
				}
				class o8 extends o6 {
					getTypeName() {
						return "HTML";
					}
				}
				class o7 extends o6 {
					getTypeName() {
						return "Style";
					}
				}
				class o9 extends o6 {
					getTypeName() {
						return "Script";
					}
				}
				class ie extends o6 {
					getTypeName() {
						return "URL";
					}
				}
				class it extends o6 {
					getTypeName() {
						return "ResourceURL";
					}
				}
				function ir(e) {
					return e instanceof o6 ? e.changingThisBreaksApplicationSecurity : e;
				}
				function io(e, t) {
					let n = (function (e) {
						return (e instanceof o6 && e.getTypeName()) || null;
					})(e);
					if (null != n && n !== t) {
						if ("ResourceURL" === n && "URL" === t) return !0;
						throw Error(`Required a safe ${t}, got a ${n} (see ${E})`);
					}
					return n === t;
				}
				function ii(e) {
					return new o8(e);
				}
				function is(e) {
					return new o7(e);
				}
				function il(e) {
					return new o9(e);
				}
				function ia(e) {
					return new ie(e);
				}
				function iu(e) {
					return new it(e);
				}
				function id(e) {
					let t = new ih(e);
					return (function () {
						try {
							return !!new window.DOMParser().parseFromString(
								oJ(""),
								"text/html",
							);
						} catch (e) {
							return !1;
						}
					})()
						? new ic(t)
						: t;
				}
				class ic {
					getInertBodyElement(e) {
						e = "<body><remove></remove>" + e;
						try {
							let t = new window.DOMParser().parseFromString(
								oJ(e),
								"text/html",
							).body;
							if (null === t)
								return this.inertDocumentHelper.getInertBodyElement(e);
							return t.removeChild(t.firstChild), t;
						} catch (e) {
							return null;
						}
					}
					constructor(e) {
						this.inertDocumentHelper = e;
					}
				}
				class ih {
					getInertBodyElement(e) {
						let t = this.inertDocument.createElement("template");
						return (t.innerHTML = oJ(e)), t;
					}
					constructor(e) {
						(this.defaultDoc = e),
							(this.inertDocument =
								this.defaultDoc.implementation.createHTMLDocument(
									"sanitization-inert",
								));
					}
				}
				let ip = /^(?!javascript:)(?:[a-z0-9+.-]+:|[^&:\/?#]*(?:[\/?#]|$))/i;
				function im(e) {
					return (e = String(e)).match(ip)
						? e
						: (("undefined" == typeof ngDevMode || ngDevMode) &&
								console.warn(
									`WARNING: sanitizing unsafe URL value ${e} (see ${E})`,
								),
						  "unsafe:" + e);
				}
				function ig(e) {
					let t = {};
					for (let n of e.split(",")) t[n] = !0;
					return t;
				}
				function iv(...e) {
					let t = {};
					for (let n of e) for (let e in n) n.hasOwnProperty(e) && (t[e] = !0);
					return t;
				}
				let iy = ig("area,br,col,hr,img,wbr"),
					ib = ig("colgroup,dd,dt,li,p,tbody,td,tfoot,th,thead,tr"),
					i_ = ig("rp,rt"),
					ij = iv(i_, ib),
					ix = iv(
						ib,
						ig(
							"address,article,aside,blockquote,caption,center,del,details,dialog,dir,div,dl,figure,figcaption,footer,h1,h2,h3,h4,h5,h6,header,hgroup,hr,ins,main,map,menu,nav,ol,pre,section,summary,table,ul",
						),
					),
					iD = iv(
						i_,
						ig(
							"a,abbr,acronym,audio,b,bdi,bdo,big,br,cite,code,del,dfn,em,font,i,img,ins,kbd,label,map,mark,picture,q,ruby,rp,rt,s,samp,small,source,span,strike,strong,sub,sup,time,track,tt,u,var,video",
						),
					),
					iw = iv(iy, ix, iD, ij),
					iM = ig(
						"background,cite,href,itemtype,longdesc,poster,src,xlink:href",
					),
					iC = ig(
						"abbr,accesskey,align,alt,autoplay,axis,bgcolor,border,cellpadding,cellspacing,class,clear,color,cols,colspan,compact,controls,coords,datetime,default,dir,download,face,headers,height,hidden,hreflang,hspace,ismap,itemscope,itemprop,kind,label,lang,language,loop,media,muted,nohref,nowrap,open,preload,rel,rev,role,rows,rowspan,rules,scope,scrolling,shape,size,sizes,span,srclang,srcset,start,summary,tabindex,target,title,translate,type,usemap,valign,value,vspace,width",
					),
					iS = ig(
						"aria-activedescendant,aria-atomic,aria-autocomplete,aria-busy,aria-checked,aria-colcount,aria-colindex,aria-colspan,aria-controls,aria-current,aria-describedby,aria-details,aria-disabled,aria-dropeffect,aria-errormessage,aria-expanded,aria-flowto,aria-grabbed,aria-haspopup,aria-hidden,aria-invalid,aria-keyshortcuts,aria-label,aria-labelledby,aria-level,aria-live,aria-modal,aria-multiline,aria-multiselectable,aria-orientation,aria-owns,aria-placeholder,aria-posinset,aria-pressed,aria-readonly,aria-relevant,aria-required,aria-roledescription,aria-rowcount,aria-rowindex,aria-rowspan,aria-selected,aria-setsize,aria-sort,aria-valuemax,aria-valuemin,aria-valuenow,aria-valuetext",
					),
					iE = iv(iM, iC, iS),
					iO = ig("script,style,template");
				class iA {
					sanitizeChildren(e) {
						let t = e.firstChild,
							n = !0;
						for (; t; ) {
							if (
								(t.nodeType === Node.ELEMENT_NODE
									? (n = this.startElement(t))
									: t.nodeType === Node.TEXT_NODE
									? this.chars(t.nodeValue)
									: (this.sanitizedSomething = !0),
								n && t.firstChild)
							) {
								t = t.firstChild;
								continue;
							}
							for (; t; ) {
								t.nodeType === Node.ELEMENT_NODE && this.endElement(t);
								let e = this.checkClobberedElement(t, t.nextSibling);
								if (e) {
									t = e;
									break;
								}
								t = this.checkClobberedElement(t, t.parentNode);
							}
						}
						return this.buf.join("");
					}
					startElement(e) {
						let t = e.nodeName.toLowerCase();
						if (!iw.hasOwnProperty(t))
							return (this.sanitizedSomething = !0), !iO.hasOwnProperty(t);
						this.buf.push("<"), this.buf.push(t);
						let n = e.attributes;
						for (let e = 0; e < n.length; e++) {
							let t = n.item(e),
								r = t.name,
								o = r.toLowerCase();
							if (!iE.hasOwnProperty(o)) {
								this.sanitizedSomething = !0;
								continue;
							}
							let i = t.value;
							iM[o] && (i = im(i)), this.buf.push(" ", r, '="', iT(i), '"');
						}
						return this.buf.push(">"), !0;
					}
					endElement(e) {
						let t = e.nodeName.toLowerCase();
						iw.hasOwnProperty(t) &&
							!iy.hasOwnProperty(t) &&
							(this.buf.push("</"), this.buf.push(t), this.buf.push(">"));
					}
					chars(e) {
						this.buf.push(iT(e));
					}
					checkClobberedElement(e, t) {
						if (
							t &&
							(e.compareDocumentPosition(t) &
								Node.DOCUMENT_POSITION_CONTAINED_BY) ===
								Node.DOCUMENT_POSITION_CONTAINED_BY
						)
							throw Error(
								`Failed to sanitize html because the element is clobbered: ${e.outerHTML}`,
							);
						return t;
					}
					constructor() {
						(this.sanitizedSomething = !1), (this.buf = []);
					}
				}
				let iI = /[\uD800-\uDBFF][\uDC00-\uDFFF]/g,
					iP = /([^\#-~ |!])/g;
				function iT(e) {
					return e
						.replace(/&/g, "&amp;")
						.replace(iI, function (e) {
							let t = e.charCodeAt(0),
								n = e.charCodeAt(1);
							return "&#" + ((t - 55296) * 1024 + (n - 56320) + 65536) + ";";
						})
						.replace(iP, function (e) {
							return "&#" + e.charCodeAt(0) + ";";
						})
						.replace(/</g, "&lt;")
						.replace(/>/g, "&gt;");
				}
				function ik(e, t) {
					let n = null;
					try {
						a = a || id(e);
						let r = t ? String(t) : "";
						n = a.getInertBodyElement(r);
						let o = 5,
							i = r;
						do {
							if (0 === o)
								throw Error(
									"Failed to sanitize html because the input is unstable",
								);
							o--, (r = i), (i = n.innerHTML), (n = a.getInertBodyElement(r));
						} while (r !== i);
						let s = new iA(),
							l = s.sanitizeChildren(iF(n) || n);
						return (
							("undefined" == typeof ngDevMode || ngDevMode) &&
								s.sanitizedSomething &&
								console.warn(
									`WARNING: sanitizing HTML stripped some content, see ${E}`,
								),
							oJ(l)
						);
					} finally {
						if (n) {
							let e = iF(n) || n;
							for (; e.firstChild; ) e.removeChild(e.firstChild);
						}
					}
				}
				function iF(e) {
					return "content" in e &&
						(function (e) {
							return (
								e.nodeType === Node.ELEMENT_NODE && "TEMPLATE" === e.nodeName
							);
						})(e)
						? e.content
						: null;
				}
				var iR =
					(((iR = iR || {})[(iR.NONE = 0)] = "NONE"),
					(iR[(iR.HTML = 1)] = "HTML"),
					(iR[(iR.STYLE = 2)] = "STYLE"),
					(iR[(iR.SCRIPT = 3)] = "SCRIPT"),
					(iR[(iR.URL = 4)] = "URL"),
					(iR[(iR.RESOURCE_URL = 5)] = "RESOURCE_URL"),
					iR);
				function iN(e) {
					let t = iW();
					return t
						? o2(t.sanitize(iR.HTML, e) || "")
						: io(e, "HTML")
						? o2(ir(e))
						: ik(o1(), I(e));
				}
				function iL(e) {
					let t = iW();
					return t
						? t.sanitize(iR.STYLE, e) || ""
						: io(e, "Style")
						? ir(e)
						: I(e);
				}
				function i$(e) {
					let t = iW();
					return t
						? t.sanitize(iR.URL, e) || ""
						: io(e, "URL")
						? ir(e)
						: im(I(e));
				}
				function iV(e) {
					let t = iW();
					if (t) return o4(t.sanitize(iR.RESOURCE_URL, e) || "");
					if (io(e, "ResourceURL")) return o4(ir(e));
					throw new O(
						904,
						ngDevMode &&
							`unsafe value used in a resource URL context (see ${E})`,
					);
				}
				function iB(e) {
					let t = iW();
					if (t) return o3(t.sanitize(iR.SCRIPT, e) || "");
					if (io(e, "Script")) return o3(ir(e));
					throw new O(
						905,
						ngDevMode && "unsafe value used in a script context",
					);
				}
				function iU(e) {
					if (
						ngDevMode &&
						(!Array.isArray(e) || !Array.isArray(e.raw) || 1 !== e.length)
					)
						throw Error(
							`Unexpected interpolation in trusted HTML constant: ${e.join(
								"?",
							)}`,
						);
					return oJ(e[0]);
				}
				function iH(e) {
					var t, n;
					if (
						ngDevMode &&
						(!Array.isArray(e) || !Array.isArray(e.raw) || 1 !== e.length)
					)
						throw Error(
							`Unexpected interpolation in trusted URL constant: ${e.join(
								"?",
							)}`,
						);
					return (
						(t = e[0]),
						(null === (n = oK()) || void 0 === n
							? void 0
							: n.createScriptURL(t)) || t
					);
				}
				function iz(e, t, n) {
					var r, o;
					return ((r = t),
					("src" === (o = n) &&
						("embed" === r ||
							"frame" === r ||
							"iframe" === r ||
							"media" === r ||
							"script" === r)) ||
					("href" === o && ("base" === r || "link" === r))
						? iV
						: i$)(e);
				}
				function iW() {
					let e = t9();
					return e && e[10].sanitizer;
				}
				class iq {
					get multi() {
						return this;
					}
					toString() {
						return `InjectionToken ${this._desc}`;
					}
					constructor(e, t) {
						(this._desc = e),
							(this.ngMetadataName = "InjectionToken"),
							(this.ɵprov = void 0),
							"number" == typeof t
								? (("undefined" == typeof ngDevMode || ngDevMode) &&
										z(t, 0, "Only negative numbers are supported here"),
								  (this.__NG_ELEMENT_ID__ = t))
								: void 0 !== t &&
								  (this.ɵprov = J({
										token: this,
										providedIn: t.providedIn || "root",
										factory: t.factory,
								  }));
					}
				}
				let iG = new iq("ENVIRONMENT_INITIALIZER"),
					iZ = new iq("INJECTOR", -1),
					iY = new iq("INJECTOR_DEF_TYPES");
				class iQ {
					get(e, t = eh) {
						if (t === eh) {
							let t = Error(`NullInjectorError: No provider for ${j(e)}!`);
							throw ((t.name = "NullInjectorError"), t);
						}
						return t;
					}
				}
				function iK(e) {
					return { ɵproviders: e };
				}
				function iJ(e, ...t) {
					let n;
					let r = [],
						o = new Set();
					return (
						rw(t, (t) => {
							if (("undefined" == typeof ngDevMode || ngDevMode) && e) {
								let e = e6(t);
								if (null == e ? void 0 : e.standalone)
									throw new O(
										800,
										`Importing providers supports NgModule or ModuleWithProviders but got a standalone component "${P(
											t,
										)}"`,
									);
							}
							(function e(t, n, r, o) {
								if (!(t = M(t))) return !1;
								let i = null,
									s = er(t),
									l = !s && e6(t);
								if (s || l) {
									if (l && !l.standalone) return !1;
									i = t;
								} else {
									let e = t.ngModule;
									if (!(s = er(e))) return !1;
									i = e;
								}
								if (ngDevMode && -1 !== r.indexOf(i)) {
									let e = j(i),
										t = r.map(j);
									T(e, t);
								}
								let a = o.has(i);
								if (l) {
									if (a) return !1;
									if ((o.add(i), l.dependencies)) {
										let t =
											"function" == typeof l.dependencies
												? l.dependencies()
												: l.dependencies;
										for (let i of t) e(i, n, r, o);
									}
								} else {
									if (!s) return !1;
									if (null != s.imports && !a) {
										let t;
										ngDevMode && r.push(i), o.add(i);
										try {
											rw(s.imports, (i) => {
												e(i, n, r, o) && (t || (t = []), t.push(i));
											});
										} finally {
											ngDevMode && r.pop();
										}
										void 0 !== t && iX(t, n);
									}
									if (!a) {
										let e = tM(i) || (() => new i());
										n.push(
											{ provide: i, useFactory: e, deps: eA },
											{ provide: iY, useValue: i, multi: !0 },
											{ provide: iG, useValue: () => e_(i), multi: !0 },
										);
									}
									let l = s.providers;
									if (null != l && !a) {
										let e = t;
										i1(l, (t) => {
											ngDevMode && i0(t, l, e), n.push(t);
										});
									}
								}
								return i !== t && void 0 !== t.providers;
							})(t, r, [], o) && (n || (n = []), n.push(t));
						}),
						void 0 !== n && iX(n, r),
						r
					);
				}
				function iX(e, t) {
					for (let n = 0; n < e.length; n++) {
						let { ngModule: r, providers: o } = e[n];
						i1(o, (e) => {
							ngDevMode && i0(e, o || eA, r), t.push(e);
						});
					}
				}
				function i0(e, t, n) {
					if (i6(e) || i2(e) || i4(e) || i3(e)) return;
					let r = M(e && (e.useClass || e.provide));
					!r && F(n, t, e);
				}
				function i1(e, t) {
					for (let n of e)
						S(n) && (n = n.ɵproviders), Array.isArray(n) ? i1(n, t) : t(n);
				}
				let i5 = b({ provide: String, useValue: b });
				function i2(e) {
					return null !== e && "object" == typeof e && i5 in e;
				}
				function i3(e) {
					return !!(e && e.useExisting);
				}
				function i4(e) {
					return !!(e && e.useFactory);
				}
				function i6(e) {
					return "function" == typeof e;
				}
				let i8 = new iq("Set Injector scope."),
					i7 = {},
					i9 = {};
				function se() {
					return void 0 === f && (f = new iQ()), f;
				}
				class st {}
				class sn extends st {
					get destroyed() {
						return this._destroyed;
					}
					destroy() {
						this.assertNotDestroyed(), (this._destroyed = !0);
						try {
							for (let e of this._ngOnDestroyHooks) e.ngOnDestroy();
							for (let e of this._onDestroyHooks) e();
						} finally {
							this.records.clear(),
								this._ngOnDestroyHooks.clear(),
								this.injectorDefTypes.clear(),
								(this._onDestroyHooks.length = 0);
						}
					}
					onDestroy(e) {
						return (
							this.assertNotDestroyed(),
							this._onDestroyHooks.push(e),
							() => this.removeOnDestroy(e)
						);
					}
					runInContext(e) {
						this.assertNotDestroyed();
						let t = ey(this),
							n = eu(void 0);
						try {
							return e();
						} finally {
							ey(t), eu(n);
						}
					}
					get(e, t = eh, n = ea.Default) {
						if ((this.assertNotDestroyed(), e.hasOwnProperty(eN)))
							return e[eN](this);
						n = eD(n);
						let r = ey(this),
							o = eu(void 0);
						try {
							if (!(n & ea.SkipSelf)) {
								let t = this.records.get(e);
								if (void 0 === t) {
									let n =
										(function (e) {
											return (
												"function" == typeof e ||
												("object" == typeof e && e instanceof iq)
											);
										})(e) && ee(e);
									(t =
										n && this.injectableDefInScope(n) ? si(sr(e), i7) : null),
										this.records.set(e, t);
								}
								if (null != t) return this.hydrate(e, t);
							}
							let r = n & ea.Self ? se() : this.parent;
							return (t = n & ea.Optional && t === eh ? null : t), r.get(e, t);
						} catch (t) {
							if ("NullInjectorError" === t.name) {
								let n = (t[em] = t[em] || []);
								if ((n.unshift(j(e)), !r))
									return (function (e, t, n, r) {
										let o = e[em];
										throw (
											(t[ev] && o.unshift(t[ev]),
											(e.message = (function (e, t, n, r = null) {
												e =
													e && "\n" === e.charAt(0) && "ɵ" == e.charAt(1)
														? e.slice(2)
														: e;
												let o = j(t);
												if (Array.isArray(t)) o = t.map(j).join(" -> ");
												else if ("object" == typeof t) {
													let e = [];
													for (let n in t)
														if (t.hasOwnProperty(n)) {
															let r = t[n];
															e.push(
																n +
																	":" +
																	("string" == typeof r
																		? JSON.stringify(r)
																		: j(r)),
															);
														}
													o = `{${e.join(", ")}}`;
												}
												return `${n}${
													r ? "(" + r + ")" : ""
												}[${o}]: ${e.replace(eg, "\n  ")}`;
											})("\n" + e.message, o, n, r)),
											(e.ngTokenPath = o),
											(e[em] = null),
											e)
										);
									})(t, e, "R3InjectorError", this.source);
							}
							throw t;
						} finally {
							eu(o), ey(r);
						}
					}
					resolveInjectorInitializers() {
						let e = ey(this),
							t = eu(void 0);
						try {
							let e = this.get(iG.multi, eA, ea.Self);
							if (ngDevMode && !Array.isArray(e))
								throw new O(
									-209,
									`Unexpected type of the \`ENVIRONMENT_INITIALIZER\` token value (expected an array, but got ${typeof e}). Please check that the \`ENVIRONMENT_INITIALIZER\` token is configured as a \`multi: true\` provider.`,
								);
							for (let t of e) t();
						} finally {
							ey(e), eu(t);
						}
					}
					toString() {
						let e = [],
							t = this.records;
						for (let n of t.keys()) e.push(j(n));
						return `R3Injector[${e.join(", ")}]`;
					}
					assertNotDestroyed() {
						if (this._destroyed)
							throw new O(
								205,
								ngDevMode && "Injector has already been destroyed.",
							);
					}
					processProvider(e) {
						let t = i6((e = M(e))) ? e : M(e && e.provide),
							n = (function (e) {
								if (i2(e)) return si(void 0, e.useValue);
								{
									let t = so(e);
									return si(t, i7);
								}
							})(e);
						if (i6(e) || !0 !== e.multi) {
							let e = this.records.get(t);
							ngDevMode && e && void 0 !== e.multi && k();
						} else {
							let n = this.records.get(t);
							n
								? ngDevMode && void 0 === n.multi && k()
								: (((n = si(void 0, i7, !0)).factory = () => ew(n.multi)),
								  this.records.set(t, n)),
								(t = e),
								n.multi.push(e);
						}
						this.records.set(t, n);
					}
					hydrate(e, t) {
						return (
							ngDevMode && t.value === i9
								? T(j(e))
								: t.value === i7 && ((t.value = i9), (t.value = t.factory())),
							"object" == typeof t.value &&
								t.value &&
								(function (e) {
									return (
										null !== e &&
										"object" == typeof e &&
										"function" == typeof e.ngOnDestroy
									);
								})(t.value) &&
								this._ngOnDestroyHooks.add(t.value),
							t.value
						);
					}
					injectableDefInScope(e) {
						if (!e.providedIn) return !1;
						let t = M(e.providedIn);
						return "string" == typeof t
							? "any" === t || this.scopes.has(t)
							: this.injectorDefTypes.has(t);
					}
					removeOnDestroy(e) {
						let t = this._onDestroyHooks.indexOf(e);
						-1 !== t && this._onDestroyHooks.splice(t, 1);
					}
					constructor(e, t, n, r) {
						super(),
							(this.parent = t),
							(this.source = n),
							(this.scopes = r),
							(this.records = new Map()),
							(this._ngOnDestroyHooks = new Set()),
							(this._onDestroyHooks = []),
							(this._destroyed = !1),
							(function e(t, n) {
								for (let r of t)
									Array.isArray(r)
										? e(r, n)
										: r && S(r)
										? e(r.ɵproviders, n)
										: n(r);
							})(e, (e) => this.processProvider(e)),
							this.records.set(iZ, si(void 0, this)),
							r.has("environment") && this.records.set(st, si(void 0, this));
						let o = this.records.get(i8);
						null != o && "string" == typeof o.value && this.scopes.add(o.value),
							(this.injectorDefTypes = new Set(
								this.get(iY.multi, eA, ea.Self),
							));
					}
				}
				function sr(e) {
					let t = ee(e),
						n = null !== t ? t.factory : tM(e);
					if (null !== n) return n;
					if (e instanceof iq)
						throw new O(
							204,
							ngDevMode && `Token ${j(e)} is missing a ɵprov definition.`,
						);
					if (e instanceof Function)
						return (function (e) {
							let t = e.length;
							if (t > 0) {
								let n = rS(t, "?");
								throw new O(
									204,
									ngDevMode &&
										`Can't resolve all parameters for ${j(e)}: (${n.join(
											", ",
										)}).`,
								);
							}
							let n = (function (e) {
								let t = e && (e[eo] || e[es]);
								return t
									? (ngDevMode &&
											console.warn(`DEPRECATED: DI is instantiating a token "${e.name}" that inherits its @Injectable decorator but does not provide one itself.
This will become an error in a future version of Angular. Please add @Injectable() to the "${e.name}" class.`),
									  t)
									: null;
							})(e);
							return null !== n ? () => n.factory(e) : () => new e();
						})(e);
					throw new O(204, ngDevMode && "unreachable");
				}
				function so(e, t, n) {
					let r;
					if ((ngDevMode && S(e) && F(void 0, n, e), i6(e))) {
						let t = M(e);
						return tM(t) || sr(t);
					}
					if (i2(e)) r = () => M(e.useValue);
					else if (i4(e)) r = () => e.useFactory(...ew(e.deps || []));
					else if (i3(e)) r = () => e_(M(e.useExisting));
					else {
						let o = M(e && (e.useClass || e.provide));
						if (
							(ngDevMode && !o && F(t, n, e),
							!(function (e) {
								return !!e.deps;
							})(e))
						)
							return tM(o) || sr(o);
						r = () => new o(...ew(e.deps));
					}
					return r;
				}
				function si(e, t, n = !1) {
					return { factory: e, value: t, multi: n ? [] : void 0 };
				}
				let ss = new iq("AppId", { providedIn: "root", factory: () => sl }),
					sl = "ng",
					sa = new iq("Platform Initializer"),
					su = new iq("Platform ID", {
						providedIn: "platform",
						factory: () => "unknown",
					});
				new iq("Application Packages Root URL"), new iq("AnimationModuleType");
				let sd = new iq("CSP nonce", {
					providedIn: "root",
					factory: () => {
						var e, t;
						return (
							(null ===
								(e =
									null === (t = o1().body) || void 0 === t
										? void 0
										: t.querySelector("[ngCspNonce]")) || void 0 === e
								? void 0
								: e.getAttribute("ngCspNonce")) || null
						);
					},
				});
				new iq(
					"undefined" == typeof ngDevMode || ngDevMode
						? "ENABLED_SSR_FEATURES"
						: "",
					{ providedIn: "root", factory: () => new Set() },
				);
				function sc(e) {
					return e;
				}
				function sf() {
					let e = new sh();
					return (
						(e.store = (function (e, t) {
							let n = e.getElementById(t + "-state"),
								r = {};
							if (n && n.textContent)
								try {
									r = JSON.parse(
										(function (e) {
											let t = {
												"&a;": "&",
												"&q;": '"',
												"&s;": "'",
												"&l;": "<",
												"&g;": ">",
											};
											return e.replace(/&[^;]+;/g, (e) => t[e]);
										})(n.textContent),
									);
								} catch (e) {
									console.warn(
										"Exception while restoring TransferState for app " + t,
										e,
									);
								}
							return r;
						})(o1(), ex(ss))),
						e
					);
				}
				let sh = (() => {
					class e {
						get(e, t) {
							return void 0 !== this.store[e] ? this.store[e] : t;
						}
						set(e, t) {
							this.store[e] = t;
						}
						remove(e) {
							delete this.store[e];
						}
						hasKey(e) {
							return this.store.hasOwnProperty(e);
						}
						get isEmpty() {
							return 0 === Object.keys(this.store).length;
						}
						onSerialize(e, t) {
							this.onSerializeCallbacks[e] = t;
						}
						toJson() {
							for (let e in this.onSerializeCallbacks)
								if (this.onSerializeCallbacks.hasOwnProperty(e))
									try {
										this.store[e] = this.onSerializeCallbacks[e]();
									} catch (e) {
										console.warn("Exception in onSerialize callback: ", e);
									}
							return JSON.stringify(this.store);
						}
						constructor() {
							(this.store = {}), (this.onSerializeCallbacks = {});
						}
					}
					return (
						(e.ɵprov = J({ token: e, providedIn: "root", factory: sf })), e
					);
				})();
				var sp =
					(((sp = sp || {}).FirstChild = "f"), (sp.NextSibling = "n"), sp);
				let sm = (e, t) => null;
				function sg(e, t) {
					return sm(e, t);
				}
				class sv {}
				class sy {}
				let sb = "ngComponent";
				class s_ {
					resolveComponentFactory(e) {
						throw (function (e) {
							let t = Error(`No component factory found for ${j(e)}.`);
							return (t[sb] = e), t;
						})(e);
					}
				}
				let sj = (() => {
					class e {}
					return (e.NULL = new s_()), e;
				})();
				function sx() {
					return sD(nr(), t9());
				}
				function sD(e, t) {
					return new sw(tG(e, t));
				}
				let sw = (() => {
					class e {
						constructor(e) {
							this.nativeElement = e;
						}
					}
					return (e.__NG_ELEMENT_ID__ = sx), e;
				})();
				function sM(e) {
					return e instanceof sw ? e.nativeElement : e;
				}
				class sC {}
				let sS = (() => {
						class e {}
						return (
							(e.__NG_ELEMENT_ID__ = () =>
								(function () {
									let e = t9(),
										t = nr(),
										n = tQ(t.index, e);
									return (ti(n) ? n : e)[11];
								})()),
							e
						);
					})(),
					sE = (() => {
						class e {}
						return (
							(e.ɵprov = J({
								token: e,
								providedIn: "root",
								factory: () => null,
							})),
							e
						);
					})();
				class sO {
					constructor(e) {
						(this.full = e),
							(this.major = e.split(".")[0]),
							(this.minor = e.split(".")[1]),
							(this.patch = e.split(".").slice(2).join("."));
					}
				}
				let sA = new sO("16.0.0"),
					sI = {};
				function sP(e) {
					for (; e; ) {
						e[2] |= 64;
						let t = ob(e);
						if ((512 & e[2]) != 0 && !t) return e;
						e = t;
					}
					return null;
				}
				function sT(e) {
					return e.ngOriginalError;
				}
				class sk {
					handleError(e) {
						let t = this._findOriginalError(e);
						this._console.error("ERROR", e),
							t && this._console.error("ORIGINAL ERROR", t);
					}
					_findOriginalError(e) {
						let t = e && sT(e);
						for (; t && sT(t); ) t = sT(t);
						return t || null;
					}
					constructor() {
						this._console = console;
					}
				}
				new iq(
					"undefined" == typeof ngDevMode || ngDevMode
						? "IS_HYDRATION_DOM_REUSE_ENABLED"
						: "",
				);
				let sF = new iq(
						"undefined" == typeof ngDevMode || ngDevMode
							? "PRESERVE_HOST_CONTENT"
							: "",
						{ providedIn: "root", factory: () => !1 },
					),
					sR = /([A-Z])/g;
				function sN(e) {
					return e.ownerDocument.defaultView;
				}
				function sL(e) {
					return e.ownerDocument;
				}
				function s$(e) {
					return e.ownerDocument.body;
				}
				let sV = `�`;
				function sB(e) {
					return e instanceof Function ? e() : e;
				}
				function sU(e, t, n, r, o) {
					let [i, s, ...l] = r.split(sV),
						a = s,
						u = s;
					for (let r = 0; r < l.length; r++) {
						let i = t + r;
						(a += `${e[i]}${l[r]}`), (u += `${i === n ? o : e[i]}${l[r]}`);
					}
					return { propName: i, oldValue: a, newValue: u };
				}
				class sH extends tI {
					set lView(e) {
						("undefined" == typeof ngDevMode || ngDevMode) &&
							V(this._lView, null, "Consumer already associated with a view."),
							(this._lView = e);
					}
					onConsumerDependencyMayHaveChanged() {
						("undefined" == typeof ngDevMode || ngDevMode) &&
							Z(
								this._lView,
								"Updating a signal during template or host binding execution is not allowed.",
							),
							sP(this._lView);
					}
					onProducerUpdateValueVersion() {}
					get hasReadASignal() {
						return this.hasProducers;
					}
					runInContext(e, t, n) {
						let r = tA(this);
						this.trackingVersion++;
						try {
							e(t, n);
						} finally {
							tA(r);
						}
					}
					destroy() {
						this.trackingVersion++;
					}
					constructor() {
						super(...arguments),
							(this.consumerAllowSignalWrites = !1),
							(this._lView = null);
					}
				}
				let sz = null;
				function sW() {
					return null != sz || (sz = new sH()), sz;
				}
				function sq(e, t) {
					var n;
					return null !== (n = e[t]) && void 0 !== n ? n : sW();
				}
				function sG(e, t) {
					let n = sW();
					n.hasReadASignal && ((e[t] = sz), (n.lView = e), (sz = new sH()));
				}
				let sZ =
					"undefined" == typeof ngDevMode || ngDevMode
						? { __brand__: "NO_CHANGE" }
						: {};
				function sY(e) {
					ngDevMode && q(e, 0, "Can only advance forward"),
						sQ(ne(), t9(), nS() + e, !!ngDevMode && nu());
				}
				function sQ(e, t, n, r) {
					if (
						(ngDevMode &&
							(function (e, t) {
								let n = e[1];
								tx(25, n.bindingStartIndex, t);
							})(t, n),
						!r)
					) {
						let r = (3 & t[2]) == 3;
						if (r) {
							let r = e.preOrderCheckHooks;
							if (null !== r) nV(t, r, 3, n);
						} else {
							let r = e.preOrderHooks;
							null !== r && nL(t, r, 0, n);
						}
					}
					nE(n);
				}
				let sK = {
						ɵɵdefineInjectable: J,
						ɵɵdefineInjector: X,
						ɵɵinject: e_,
						ɵɵinvalidFactoryDep: ej,
						resolveForwardRef: M,
					},
					sJ = b({ provide: String, useValue: b });
				function sX(e) {
					return void 0 !== e.useClass;
				}
				function s0(e) {
					return void 0 !== e.useFactory;
				}
				let s1 = rc("Injectable", void 0, void 0, void 0, (e, t) => {
					var n, r;
					let o, i;
					return (
						(n = e),
						(r = t),
						(o = null),
						(i = null),
						void (!n.hasOwnProperty(eo) &&
							Object.defineProperty(n, eo, {
								get: () => {
									if (null === o) {
										let e = r_({ usage: 0, kind: "injectable", type: n });
										o = e.compileInjectable(
											sK,
											`ng:///${n.name}/ɵprov.js`,
											(function (e, t) {
												let n = t || { providedIn: null },
													r = {
														name: e.name,
														type: e,
														typeArgumentCount: 0,
														providedIn: n.providedIn,
													};
												if (
													((sX(n) || s0(n)) &&
														void 0 !== n.deps &&
														(r.deps = rG(n.deps)),
													sX(n))
												)
													r.useClass = n.useClass;
												else {
													if (sJ in n) r.useValue = n.useValue;
													else if (s0(n)) r.useFactory = n.useFactory;
													else if (void 0 !== n.useExisting)
														r.useExisting = n.useExisting;
												}
												return r;
											})(n, r),
										);
									}
									return o;
								},
							}),
						!n.hasOwnProperty(eF) &&
							Object.defineProperty(n, eF, {
								get: () => {
									if (null === i) {
										let e = r_({ usage: 0, kind: "injectable", type: n });
										i = e.compileFactory(sK, `ng:///${n.name}/ɵfac.js`, {
											name: n.name,
											type: n,
											typeArgumentCount: 0,
											deps: rq(n),
											target: e.FactoryTarget.Injectable,
										});
									}
									return i;
								},
								configurable: !0,
							}))
					);
				});
				function s5(e, t = null, n = null, r) {
					let o = s2(e, t, n, r);
					return o.resolveInjectorInitializers(), o;
				}
				function s2(e, t = null, n = null, r, o = new Set()) {
					let i = [
						n || eA,
						(function (...e) {
							return { ɵproviders: iJ(!0, e), ɵfromNgModule: !0 };
						})(e),
					];
					return (
						(r = r || ("object" == typeof e ? void 0 : j(e))),
						new sn(i, t || se(), r || null, o)
					);
				}
				let s3 = (() => {
					class e {
						static create(e, t) {
							if (Array.isArray(e)) return s5({ name: "" }, t, e, "");
							{
								var n;
								let t = null !== (n = e.name) && void 0 !== n ? n : "";
								return s5({ name: t }, e.parent, e.providers, t);
							}
						}
					}
					return (
						(e.THROW_IF_NOT_FOUND = eh),
						(e.NULL = new iQ()),
						(e.ɵprov = J({
							token: e,
							providedIn: "any",
							factory: () => e_(iZ),
						})),
						(e.__NG_ELEMENT_ID__ = -1),
						e
					);
				})();
				function s4(e, t = ea.Default) {
					let n = t9();
					if (null === n) {
						var o;
						return (
							ngDevMode &&
								((o = s4),
								ngDevMode &&
									B(r, o, "Calling ɵɵinject would cause infinite recursion")),
							e_(e, t)
						);
					}
					let i = nr();
					return n6(i, n, M(e), t);
				}
				function s6() {
					let e = ngDevMode
						? "This constructor was not compatible with Dependency Injection."
						: "invalid";
					throw Error(e);
				}
				function s8(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t.blueprint.slice();
					return (
						(c[0] = o),
						(c[2] = 140 | r),
						(null !== u || (e && 2048 & e[2])) && (c[2] |= 2048),
						(c[17] = 0),
						ngDevMode && t.declTNode && e && tc(t.declTNode, e),
						(c[3] = c[14] = e),
						(c[8] = n),
						(c[10] = s || (e && e[10])),
						ngDevMode && Z(c[10], "LViewEnvironment is required"),
						(c[11] = l || (e && e[11])),
						ngDevMode && Z(c[11], "Renderer is required"),
						(c[9] = a || (e && e[9]) || null),
						(c[6] = i),
						(c[19] = oo++),
						(c[22] = d),
						(c[20] = u),
						ngDevMode &&
							V(
								2 != t.type || null !== e,
								!0,
								"Embedded views must have parentLView",
							),
						(c[15] = 2 == t.type ? e[15] : c),
						c
					);
				}
				function s7(e, t, n, r, o) {
					var i;
					ngDevMode &&
						0 !== t &&
						G(t, 25, "TNodes can't be in the LView header."),
						ngDevMode &&
							(2 === (i = n) ||
								1 === i ||
								4 === i ||
								8 === i ||
								32 === i ||
								16 === i ||
								64 === i ||
								Y(
									`Expected TNodeType to have only a single type selected, but got ${nH(
										i,
									)}.`,
								));
					let s = e.data[t];
					if (null === s)
						(s = s9(e, t, n, r, o)), t3.lFrame.inI18n && (s.flags |= 32);
					else if (64 & s.type) {
						(s.type = n), (s.value = r), (s.attrs = o);
						let i = ni();
						(s.injectorIndex = null === i ? -1 : i.injectorIndex),
							ngDevMode && tf(s, e),
							ngDevMode && V(t, s.index, "Expecting same index");
					}
					return ns(s, !0), s;
				}
				function s9(e, t, n, r, o) {
					let i = no(),
						s = nl(),
						l = s ? i : i && i.parent,
						a = (e.data[t] = (function (e, t, n, r, o, i) {
							ngDevMode &&
								0 !== r &&
								G(r, 25, "TNodes can't be in the LView header."),
								ngDevMode &&
									H(i, void 0, "'undefined' is not valid value for 'attrs'"),
								ngDevMode && ngDevMode.tNode++,
								ngDevMode && t && tf(t, e);
							let s = t ? t.injectorIndex : -1,
								l = {
									type: n,
									index: r,
									insertBeforeIndex: null,
									injectorIndex: s,
									directiveStart: -1,
									directiveEnd: -1,
									directiveStylingLast: -1,
									componentOffset: -1,
									propertyBindings: null,
									flags: 0,
									providerIndexes: 0,
									value: o,
									attrs: i,
									mergedAttrs: null,
									localNames: null,
									initialInputs: void 0,
									inputs: null,
									outputs: null,
									tView: null,
									next: null,
									prev: null,
									projectionNext: null,
									child: null,
									parent: t,
									projection: null,
									styles: null,
									stylesWithoutHost: null,
									residualStyles: void 0,
									classes: null,
									classesWithoutHost: null,
									residualClasses: void 0,
									classBindings: 0,
									styleBindings: 0,
								};
							return ngDevMode && Object.seal(l), l;
						})(e, l, n, t, r, o));
					return (
						null === e.firstChild && (e.firstChild = a),
						null !== i &&
							(s
								? null == i.child && null !== a.parent && (i.child = a)
								: null === i.next && ((i.next = a), (a.prev = i))),
						a
					);
				}
				function le(e, t, n, r) {
					if (0 === n) return -1;
					ngDevMode &&
						(tb(e),
						U(e, t[1], "`LView` must be associated with `TView`!"),
						V(
							e.data.length,
							t.length,
							"Expecting LView to be same size as TView",
						),
						V(
							e.data.length,
							e.blueprint.length,
							"Expecting Blueprint to be same size as TView",
						),
						t_(e));
					let o = t.length;
					for (let o = 0; o < n; o++)
						t.push(r), e.blueprint.push(r), e.data.push(null);
					return o;
				}
				function lt(e, t, n, r, o) {
					let i = sq(t, 23),
						s = nS(),
						l = 2 & r;
					try {
						nE(-1), l && t.length > 25 && sQ(e, t, 25, !!ngDevMode && nu());
						if ((tH(l ? 2 : 0, o), l)) i.runInContext(n, r, o);
						else {
							let e = tA(null);
							try {
								n(r, o);
							} finally {
								tA(e);
							}
						}
					} finally {
						l && null === t[23] && sG(t, 23), nE(s);
						tH(l ? 3 : 1, o);
					}
				}
				function ln(e, t, n) {
					if (tl(t)) {
						let r = tA(null);
						try {
							let r = t.directiveStart,
								o = t.directiveEnd;
							for (let t = r; t < o; t++) {
								let r = e.data[t];
								r.contentQueries && r.contentQueries(1, n[t], t);
							}
						} finally {
							tA(r);
						}
					}
				}
				function lr(e, t, n) {
					t6() &&
						((function (e, t, n, r) {
							let o = n.directiveStart,
								i = n.directiveEnd;
							ta(n) &&
								(ngDevMode && nz(n, 3),
								(function (e, t, n) {
									let r = tG(t, e),
										o = li(n),
										i = e[10].rendererFactory,
										s = lj(
											e,
											s8(
												e,
												o,
												null,
												n.onPush ? 64 : 16,
												r,
												t,
												null,
												i.createRenderer(r, n),
												null,
												null,
												null,
											),
										);
									e[t.index] = s;
								})(t, n, e.data[o + n.componentOffset])),
								!e.firstCreatePass && nX(n, t),
								oc(r, t);
							let s = n.initialInputs;
							for (let r = o; r < i; r++) {
								let i = e.data[r],
									l = n9(t, e, r, n);
								if (
									(oc(l, t),
									null !== s &&
										(function (e, t, n, r, o, i) {
											let s = i[t];
											if (null !== s)
												for (let t = 0; t < s.length; ) {
													let i = s[t++],
														l = s[t++],
														a = s[t++];
													if ((ly(r, n, i, l, a), ngDevMode)) {
														let t = tG(o, e);
														lc(e, t, o.type, l, a);
													}
												}
										})(t, r - o, l, i, n, s),
									td(i))
								) {
									let o = tQ(n.index, t);
									o[8] = n9(t, e, r, n);
								}
							}
						})(e, t, n, tG(n, t)),
						(64 & n.flags) == 64 && lp(e, t, n));
				}
				function lo(e, t, n = tG) {
					let r = t.localNames;
					if (null !== r) {
						let o = t.index + 1;
						for (let i = 0; i < r.length; i += 2) {
							let s = r[i + 1],
								l = -1 === s ? n(t, e) : e[s];
							e[o++] = l;
						}
					}
				}
				function li(e) {
					let t = e.tView;
					return null === t || t.incompleteFirstPass
						? (e.tView = ls(
								1,
								null,
								e.template,
								e.decls,
								e.vars,
								e.directiveDefs,
								e.pipeDefs,
								e.viewQuery,
								e.schemas,
								e.consts,
								e.id,
						  ))
						: t;
				}
				function ls(e, t, n, r, o, i, s, l, a, u, d) {
					ngDevMode && ngDevMode.tView++;
					let c = 25 + r,
						f = c + o,
						h = (function (e, t) {
							let n = [];
							for (let r = 0; r < t; r++) n.push(r < e ? null : sZ);
							return n;
						})(c, f),
						p = "function" == typeof u ? u() : u,
						m = (h[1] = {
							type: e,
							blueprint: h,
							template: n,
							queries: null,
							viewQuery: l,
							declTNode: t,
							data: h.slice().fill(null, c),
							bindingStartIndex: c,
							expandoStartIndex: f,
							hostBindingOpCodes: null,
							firstCreatePass: !0,
							firstUpdatePass: !0,
							staticViewQueries: !1,
							staticContentQueries: !1,
							preOrderHooks: null,
							preOrderCheckHooks: null,
							contentHooks: null,
							contentCheckHooks: null,
							viewHooks: null,
							viewCheckHooks: null,
							destroyHooks: null,
							cleanup: null,
							contentQueries: null,
							components: null,
							directiveRegistry: "function" == typeof i ? i() : i,
							pipeRegistry: "function" == typeof s ? s() : s,
							firstChild: null,
							schemas: a,
							consts: p,
							incompleteFirstPass: !1,
							ssrId: d,
						});
					return ngDevMode && Object.seal(m), m;
				}
				let ll = (e) => null;
				function la(e, t, n, r) {
					for (let o in e)
						if (e.hasOwnProperty(o)) {
							n = null === n ? {} : n;
							let i = e[o];
							null === r
								? lu(n, t, o, i)
								: r.hasOwnProperty(o) && lu(n, t, r[o], i);
						}
					return n;
				}
				function lu(e, t, n, r) {
					e.hasOwnProperty(n) ? e[n].push(t, r) : (e[n] = [t, r]);
				}
				function ld(e, t, n, r, o, i, s, l) {
					let a;
					ngDevMode && H(o, sZ, "Incoming value should never be NO_CHANGE.");
					let u = tG(t, n),
						d = t.inputs;
					if (!l && null != d && (a = d[r]))
						lE(e, n, a, r, o),
							ta(t) &&
								(function (e, t) {
									ngDevMode && ty(e);
									let n = tQ(t, e);
									!(16 & n[2]) && (n[2] |= 64);
								})(n, t.index),
							ngDevMode &&
								(function (e, t, n, r, o) {
									if (7 & n)
										for (let i = 0; i < r.length; i += 2)
											lc(e, t, n, r[i + 1], o);
								})(n, u, t.type, a, o);
					else if (3 & t.type) {
						var c, f, h, p, m;
						if (
							((r =
								"class" === (c = r)
									? "className"
									: "for" === c
									? "htmlFor"
									: "formaction" === c
									? "formAction"
									: "innerHtml" === c
									? "innerHTML"
									: "readonly" === c
									? "readOnly"
									: "tabindex" === c
									? "tabIndex"
									: c),
							ngDevMode)
						) {
							if (
								(!(function (e) {
									if (e.toLowerCase().startsWith("on")) {
										let t = `Binding to event property '${e}' is disallowed for security reasons, please use (${e.slice(
											2,
										)})=...
If '${e}' is a directive input, make sure the directive is imported by the current module.`;
										throw new O(306, t);
									}
								})(r),
								(f = u),
								(h = r),
								(p = t.value),
								!(null === (m = e.schemas) || r7(m, p) || h in f || eB(h)) &&
									"undefined" != typeof Node &&
									null !== Node &&
									f instanceof Node)
							)
								r5(r, t.value, t.type, n);
							ngDevMode.rendererSetProperty++;
						}
						(o = null != s ? s(o, t.value || "", r) : o),
							i.setProperty(u, r, o);
					} else
						12 & t.type &&
							ngDevMode &&
							!r7(e.schemas, t.value) &&
							r5(r, t.value, t.type, n);
				}
				function lc(e, t, n, r, o) {
					var i;
					let s = e[11];
					(i = r),
						(r = `ng-reflect-${(i = (function (e) {
							return e.replace(sR, (...e) => "-" + e[1].toLowerCase());
						})(i.replace(/[$@]/g, "_")))}`);
					let l = (function (e) {
						try {
							return null != e ? e.toString().slice(0, 30) : e;
						} catch (e) {
							return "[ERROR] Exception while trying to serialize the value";
						}
					})(o);
					if (3 & n)
						null == o ? s.removeAttribute(t, r) : s.setAttribute(t, r, l);
					else {
						let e = on(`bindings=${JSON.stringify({ [r]: l }, null, 2)}`);
						s.setValue(t, e);
					}
				}
				function lf(e, t, n, r) {
					if ((ngDevMode && tb(e), t6())) {
						let o, i;
						let s = null === r ? null : { "": -1 },
							l = (function (e, t) {
								ngDevMode && tb(e), ngDevMode && nz(t, 15);
								let n = e.directiveRegistry,
									r = null,
									o = null;
								if (n)
									for (let s = 0; s < n.length; s++) {
										let l = n[s];
										if (eG(t, l.selectors, !1)) {
											if ((r || (r = []), td(l))) {
												if (
													(ngDevMode &&
														(nz(
															t,
															2,
															`"${
																t.value
															}" tags cannot be used as component hosts. Please use a different tag to activate the ${j(
																l.type,
															)} component.`,
														),
														ta(t) &&
															!(function (e, t, n) {
																throw new O(
																	-300,
																	`Multiple components match node with tagname ${
																		e.value
																	}: ${P(t)} and ${P(n)}`,
																);
															})(t, r.find(td).type, l.type)),
													null !== l.findHostDirectiveDefs)
												) {
													let n = [];
													(o = o || new Map()),
														l.findHostDirectiveDefs(l, n, o),
														r.unshift(...n, l);
													let i = n.length;
													lm(e, t, i);
												} else r.unshift(l), lm(e, t, 0);
											} else {
												var i;
												(o = o || new Map()),
													null === (i = l.findHostDirectiveDefs) ||
														void 0 === i ||
														i.call(l, l, r, o),
													r.push(l);
											}
										}
									}
								return null === r ? null : [r, o];
							})(e, n);
						null === l ? (o = i = null) : ([o, i] = l),
							null !== o && lh(e, t, n, o, s, i),
							s &&
								(function (e, t, n) {
									if (t) {
										let r = (e.localNames = []);
										for (let e = 0; e < t.length; e += 2) {
											let o = n[t[e + 1]];
											if (null == o)
												throw new O(
													-301,
													ngDevMode &&
														`Export of name '${t[e + 1]}' not found!`,
												);
											r.push(t[e], o);
										}
									}
								})(n, r, s);
					}
					n.mergedAttrs = eU(n.mergedAttrs, n.attrs);
				}
				function lh(e, t, n, r, o, i) {
					ngDevMode && tb(e);
					for (let o = 0; o < r.length; o++) n2(nX(n, t), e, r[o].type);
					(function (e, t, n) {
						ngDevMode &&
							B(
								n,
								e.directiveEnd - e.directiveStart,
								"Reached the max number of directives",
							),
							(e.flags |= 1),
							(e.directiveStart = t),
							(e.directiveEnd = t + n),
							(e.providerIndexes = t);
					})(n, e.data.length, r.length);
					for (let e = 0; e < r.length; e++) {
						let t = r[e];
						t.providersResolver && t.providersResolver(t);
					}
					let s = !1,
						l = !1,
						a = le(e, t, r.length, null);
					ngDevMode &&
						U(
							a,
							n.directiveStart,
							"TNode.directiveStart should point to just allocated space",
						);
					for (let i = 0; i < r.length; i++) {
						var u, d;
						let c = r[i];
						(n.mergedAttrs = eU(n.mergedAttrs, c.hostAttrs)),
							(function (e, t, n, r, o) {
								ngDevMode && G(r, 25, "Must be in Expando section"),
									(e.data[r] = o);
								let i = o.factory || (o.factory = tM(o.type, !0)),
									s = new nU(i, td(o), s4);
								(e.blueprint[r] = s),
									(n[r] = s),
									!(function (e, t, n, r, o) {
										ngDevMode && tb(e);
										let i = o.hostBindings;
										if (i) {
											let o = e.hostBindingOpCodes;
											null === o && (o = e.hostBindingOpCodes = []);
											let s = ~t.index;
											(function (e) {
												let t = e.length;
												for (; t > 0; ) {
													let n = e[--t];
													if ("number" == typeof n && n < 0) return n;
												}
												return 0;
											})(o) != s && o.push(s),
												o.push(n, r, i);
										}
									})(e, t, r, le(e, n, o.hostVars, sZ), o);
							})(e, n, t, a, c),
							(function (e, t, n) {
								if (n) {
									if (t.exportAs)
										for (let r = 0; r < t.exportAs.length; r++)
											n[t.exportAs[r]] = e;
									td(t) && (n[""] = e);
								}
							})(a, c, o),
							null !== c.contentQueries && (n.flags |= 4),
							(null !== c.hostBindings ||
								null !== c.hostAttrs ||
								0 !== c.hostVars) &&
								(n.flags |= 64);
						let f = c.type.prototype;
						!s &&
							(f.ngOnChanges || f.ngOnInit || f.ngDoCheck) &&
							((null !== (u = e.preOrderHooks) && void 0 !== u
								? u
								: (e.preOrderHooks = [])
							).push(n.index),
							(s = !0)),
							!l &&
								(f.ngOnChanges || f.ngDoCheck) &&
								((null !== (d = e.preOrderCheckHooks) && void 0 !== d
									? d
									: (e.preOrderCheckHooks = [])
								).push(n.index),
								(l = !0)),
							a++;
					}
					!(function (e, t, n) {
						ngDevMode && tb(e);
						let r = t.directiveStart,
							o = t.directiveEnd,
							i = e.data,
							s = t.attrs,
							l = [],
							a = null,
							u = null;
						for (let e = r; e < o; e++) {
							let r = i[e],
								o = n ? n.get(r) : null,
								d = o ? o.inputs : null,
								c = o ? o.outputs : null;
							(a = la(r.inputs, e, a, d)), (u = la(r.outputs, e, u, c));
							let f =
								null === a || null === s || eW(t)
									? null
									: (function (e, t, n) {
											let r = null,
												o = 0;
											for (; o < n.length; ) {
												let i = n[o];
												if (0 === i) {
													o += 4;
													continue;
												}
												if (5 === i) {
													o += 2;
													continue;
												}
												if ("number" == typeof i) break;
												if (e.hasOwnProperty(i)) {
													null === r && (r = []);
													let s = e[i];
													for (let e = 0; e < s.length; e += 2)
														if (s[e] === t) {
															r.push(i, s[e + 1], n[o + 1]);
															break;
														}
												}
												o += 2;
											}
											return r;
									  })(a, e, s);
							l.push(f);
						}
						null !== a &&
							(a.hasOwnProperty("class") && (t.flags |= 8),
							a.hasOwnProperty("style") && (t.flags |= 16)),
							(t.initialInputs = l),
							(t.inputs = a),
							(t.outputs = u);
					})(e, n, i);
				}
				function lp(e, t, n) {
					let r = n.directiveStart,
						o = n.directiveEnd,
						i = n.index,
						s = t3.lFrame.currentDirectiveIndex;
					try {
						nE(i);
						for (let n = r; n < o; n++) {
							let r = e.data[n],
								o = t[n];
							nv(n),
								(null !== r.hostBindings ||
									0 !== r.hostVars ||
									null !== r.hostAttrs) &&
									(function (e, t) {
										null !== e.hostBindings && e.hostBindings(1, t);
									})(r, o);
						}
					} finally {
						nE(-1), nv(s);
					}
				}
				function lm(e, t, n) {
					var r;
					ngDevMode && tb(e),
						ngDevMode && q(n, -1, "componentOffset must be great than -1"),
						(t.componentOffset = n),
						(null !== (r = e.components) && void 0 !== r
							? r
							: (e.components = [])
						).push(t.index);
				}
				function lg(e, t, n, r, o, i) {
					ngDevMode &&
						(H(r, sZ, "Incoming value should never be NO_CHANGE."),
						!(function (e) {
							if (e.toLowerCase().startsWith("on")) {
								let t = `Binding to event attribute '${e}' is disallowed for security reasons, please use (${e.slice(
									2,
								)})=...`;
								throw new O(306, t);
							}
						})(n),
						nz(
							e,
							2,
							`Attempted to set attribute \`${n}\` on a container node. Host bindings are not valid on ng-container or ng-template.`,
						));
					let s = tG(e, t);
					lv(t[11], s, i, e.value, n, r, o);
				}
				function lv(e, t, n, r, o, i, s) {
					if (null == i)
						ngDevMode && ngDevMode.rendererRemoveAttribute++,
							e.removeAttribute(t, o, n);
					else {
						ngDevMode && ngDevMode.rendererSetAttribute++;
						let l = null == s ? I(i) : s(i, r || "", o);
						e.setAttribute(t, o, l, n);
					}
				}
				function ly(e, t, n, r, o) {
					let i = tA(null);
					try {
						null !== e.setInput ? e.setInput(t, o, n, r) : (t[r] = o);
					} finally {
						tA(i);
					}
				}
				function lb(e, t, n, r) {
					ngDevMode && ty(t);
					let o = [e, !0, !1, t, null, 0, r, n, null, null, null];
					return (
						ngDevMode &&
							V(
								o.length,
								11,
								"Should allocate correct number of slots for LContainer header.",
							),
						o
					);
				}
				function l_(e, t) {
					let n = e.contentQueries;
					if (null !== n)
						for (let r = 0; r < n.length; r += 2) {
							let o = n[r],
								i = n[r + 1];
							if (-1 !== i) {
								let n = e.data[i];
								ngDevMode && Z(n, "DirectiveDef not found."),
									ngDevMode &&
										Z(
											n.contentQueries,
											"contentQueries function should be defined",
										),
									n_(o),
									n.contentQueries(2, t[i], i);
							}
						}
				}
				function lj(e, t) {
					return e[12] ? (e[13][4] = t) : (e[12] = t), (e[13] = t), t;
				}
				function lx(e, t, n) {
					ngDevMode &&
						Z(t, "View queries function to execute must be defined."),
						n_(0);
					let r = tA(null);
					try {
						t(e, n);
					} finally {
						tA(r);
					}
				}
				function lD(e, t, n, r, ...o) {
					if (null === e[r] && (null == t.inputs || !t.inputs[n])) {
						let i = t.propertyBindings || (t.propertyBindings = []);
						i.push(r);
						let s = n;
						o.length > 0 && (s += sV + o.join(sV)), (e[r] = s);
					}
				}
				function lw(e) {
					return e[7] || (e[7] = []);
				}
				function lM(e) {
					return e.cleanup || (e.cleanup = []);
				}
				function lC(e, t, n) {
					return (
						(null === e || td(e)) &&
							(n = (function (e) {
								for (; Array.isArray(e); ) {
									if ("object" == typeof e[1]) return e;
									e = e[0];
								}
								return null;
							})(n[t.index])),
						n[11]
					);
				}
				function lS(e, t) {
					let n = e[9],
						r = n ? n.get(sk, null) : null;
					r && r.handleError(t);
				}
				function lE(e, t, n, r, o) {
					for (let i = 0; i < n.length; ) {
						let s = n[i++],
							l = n[i++],
							a = t[s];
						ngDevMode && K(t, s);
						let u = e.data[s];
						ly(u, a, r, l, o);
					}
				}
				function lO(e, t, n) {
					ngDevMode && $(n, "Value should be a string"),
						ngDevMode && H(n, sZ, "value should not be NO_CHANGE"),
						ngDevMode && K(e, t);
					let r = tq(t, e);
					ngDevMode && Z(r, "native element should exist"), oM(e[11], r, n);
				}
				function lA(e, t, n) {
					ngDevMode && V(tK(t), !0, "Should be run in creation mode"), nx(t);
					try {
						let r = e.viewQuery;
						null !== r && lx(1, r, n);
						let o = e.template;
						null !== o && lt(e, t, o, 1, n),
							e.firstCreatePass && (e.firstCreatePass = !1),
							e.staticContentQueries && l_(e, t),
							e.staticViewQueries && lx(2, e.viewQuery, n);
						let i = e.components;
						null !== i &&
							(function (e, t) {
								for (let n = 0; n < t.length; n++)
									!(function (e, t) {
										ngDevMode && V(tK(e), !0, "Should be run in creation mode");
										let n = tQ(t, e),
											r = n[1];
										(function (e, t) {
											for (let n = t.length; n < e.blueprint.length; n++)
												t.push(e.blueprint[n]);
										})(r, n);
										let o = n[0];
										if (null !== o && null === n[22]) n[22] = sm(o, n[9]);
										lA(r, n, n[8]);
									})(e, t[n]);
							})(t, i);
					} catch (t) {
						throw (
							(e.firstCreatePass &&
								((e.incompleteFirstPass = !0), (e.firstCreatePass = !1)),
							t)
						);
					} finally {
						(t[2] &= -5), nC();
					}
				}
				let lI = (() => {
					class e {}
					return (e.__NG_ELEMENT_ID__ = lT), (e.__NG_ENV_ID__ = (e) => e), e;
				})();
				class lP extends lI {
					onDestroy(e) {
						return (
							t2(this._lView, e),
							() =>
								(function (e, t) {
									if (null === e[21]) return;
									let n = e[21].indexOf(t);
									-1 !== n && e[21].splice(n, 1);
								})(this._lView, e)
						);
					}
					constructor(e) {
						super(), (this._lView = e);
					}
				}
				function lT() {
					return new lP(t9());
				}
				let lk = (() => {
					class e {
						create(e, t, n) {
							let r;
							let o = "undefined" == typeof Zone ? null : Zone.current,
								i = new tT(
									e,
									(e) => {
										this.all.has(e) && this.queue.set(e, o);
									},
									n,
								);
							this.all.add(i), i.notify();
							let s = () => {
								i.cleanup(),
									null == r || r(),
									this.all.delete(i),
									this.queue.delete(i);
							};
							return (r = null == t ? void 0 : t.onDestroy(s)), { destroy: s };
						}
						flush() {
							if (0 !== this.queue.size)
								for (let [e, t] of this.queue)
									this.queue.delete(e), t ? t.run(() => e.run()) : e.run();
						}
						get isQueueEmpty() {
							return 0 === this.queue.size;
						}
						constructor() {
							(this.all = new Set()), (this.queue = new Map());
						}
					}
					return (
						(e.ɵprov = J({
							token: e,
							providedIn: "root",
							factory: () => new e(),
						})),
						e
					);
				})();
				function lF(e, t, n) {
					ngDevMode &&
						tb(ne(), "Expecting to be called in first template pass only");
					let r = n ? e.styles : null,
						o = n ? e.classes : null,
						i = 0;
					if (null !== t)
						for (let e = 0; e < t.length; e++) {
							let n = t[e];
							if ("number" == typeof n) i = n;
							else if (1 == i) o = x(o, n);
							else if (2 == i) {
								let o = t[++e];
								r = x(r, n + ": " + o + ";");
							}
						}
					n ? (e.styles = r) : (e.stylesWithoutHost = r),
						n ? (e.classes = o) : (e.classesWithoutHost = o);
				}
				function lR(e, t, n, r = !0) {
					let o = t[10].rendererFactory,
						i = !!ngDevMode && nu();
					!i && o.begin && o.begin();
					try {
						lL(e, t, e.template, n);
					} catch (e) {
						throw (r && lS(t, e), e);
					} finally {
						var s;
						!i && o.end && o.end(),
							i ||
								null === (s = t[10].effectManager) ||
								void 0 === s ||
								s.flush();
					}
				}
				function lN(e, t, n, r = !0) {
					nd(!0);
					try {
						lR(e, t, n, r);
					} finally {
						nd(!1);
					}
				}
				function lL(e, t, n, r) {
					var o, i, s, l, a, u, d, c, f, h;
					ngDevMode && V(tK(t), !1, "Should be run in update mode");
					let p = t[2];
					if ((256 & p) == 256) return;
					let m = ngDevMode && nu();
					m || null === (o = t[10].effectManager) || void 0 === o || o.flush(),
						nx(t);
					try {
						(t[17] = 0),
							nh(e.bindingStartIndex),
							null !== n && lt(e, t, n, 2, r);
						let o = (3 & p) == 3;
						if (!m) {
							if (o) {
								let n = e.preOrderCheckHooks;
								if (null !== n) {
									(i = t), (s = n), (l = null), nV(i, s, 3, null);
								}
							} else {
								let n = e.preOrderHooks;
								null !== n && nL(t, n, 0, null), n$(t, 0);
							}
						}
						if (
							((function (e) {
								for (let n = o_(e); null !== n; n = ox(n[4])) {
									if (!n[2]) continue;
									let e = n[9];
									ngDevMode &&
										Z(e, "Transplanted View flags set but missing MOVED_VIEWS");
									for (let n = 0; n < e.length; n++) {
										var t;
										let r = e[n],
											o = r[3];
										ngDevMode && tg(o),
											(1024 & (t = r)[2]) == 0 && ((t[2] |= 1024), t5(t, 1));
									}
								}
							})(t),
							(function (e) {
								for (let t = o_(e); null !== t; t = ox(t[4]))
									for (let e = 11; e < t.length; e++) {
										let n = t[e],
											r = n[1];
										ngDevMode && Z(r, "TView must be allocated"),
											tJ(n) && lL(r, n, r.template, n[8]);
									}
							})(t),
							null !== e.contentQueries && l_(e, t),
							!m)
						) {
							if (o) {
								let n = e.contentCheckHooks;
								if (null !== n) {
									(a = t), (u = n), nV(a, u, 3, void 0);
								}
							} else {
								let n = e.contentHooks;
								null !== n && nL(t, n, 1), n$(t, 1);
							}
						}
						!(function (e, t) {
							let n = e.hostBindingOpCodes;
							if (null === n) return;
							let r = sq(t, 24);
							try {
								for (let e = 0; e < n.length; e++) {
									let o = n[e];
									if (o < 0) nE(~o);
									else {
										let i = n[++e],
											s = n[++e];
										!(function (e, t) {
											let n = t3.lFrame;
											(n.bindingIndex = n.bindingRootIndex = e), nv(t);
										})(i, o);
										let l = t[o];
										r.runInContext(s, 2, l);
									}
								}
							} finally {
								null === t[24] && sG(t, 24), nE(-1);
							}
						})(e, t);
						let d = e.components;
						null !== d &&
							(function (e, t) {
								for (let n = 0; n < t.length; n++)
									!(function (e, t) {
										ngDevMode && V(tK(e), !1, "Should be run in update mode");
										let n = tQ(t, e);
										if (tJ(n)) {
											let e = n[1];
											80 & n[2]
												? lL(e, n, e.template, n[8])
												: n[5] > 0 &&
												  (function e(t) {
														for (let n = o_(t); null !== n; n = ox(n[4]))
															for (let t = 11; t < n.length; t++) {
																let r = n[t];
																if (tJ(r)) {
																	if (1024 & r[2]) {
																		let e = r[1];
																		ngDevMode &&
																			Z(e, "TView must be allocated"),
																			lL(e, r, e.template, r[8]);
																	} else r[5] > 0 && e(r);
																}
															}
														let n = t[1],
															r = n.components;
														if (null !== r)
															for (let n = 0; n < r.length; n++) {
																let o = tQ(r[n], t);
																tJ(o) && o[5] > 0 && e(o);
															}
												  })(n);
										}
									})(e, t[n]);
							})(t, d);
						let h = e.viewQuery;
						if ((null !== h && lx(2, h, r), !m)) {
							if (o) {
								let n = e.viewCheckHooks;
								if (null !== n) {
									(c = t), (f = n), nV(c, f, 3, void 0);
								}
							} else {
								let n = e.viewHooks;
								null !== n && nL(t, n, 2), n$(t, 2);
							}
						}
						!0 === e.firstUpdatePass && (e.firstUpdatePass = !1),
							!m && (t[2] &= -73),
							t1(t);
					} finally {
						nC();
					}
				}
				class l$ {
					get rootNodes() {
						let e = this._lView,
							t = e[1];
						return (function e(t, n, r, i, s = !1) {
							for (; null !== r; ) {
								ngDevMode && nz(r, 63);
								let l = n[r.index];
								if ((null !== l && i.push(tW(l)), ts(l))) {
									for (let t = 11; t < l.length; t++) {
										let n = l[t],
											r = n[1].firstChild;
										null !== r && e(n[1], n, r, i);
									}
									l[7] !== l[0] && i.push(l[7]);
								}
								let a = r.type;
								if (8 & a) e(t, n, r.child, i);
								else if (32 & a) {
									let e;
									let t = o(r, n);
									for (; (e = t()); ) i.push(e);
								} else if (16 & a) {
									let t = oH(n, r);
									if (Array.isArray(t)) i.push(...t);
									else {
										let r = ob(n[15]);
										ngDevMode && tD(r), e(r[1], r, t, i, !0);
									}
								}
								r = s ? r.projectionNext : r.next;
							}
							return i;
						})(t, e, t.firstChild, []);
					}
					get context() {
						return this._lView[8];
					}
					set context(e) {
						this._lView[8] = e;
					}
					get destroyed() {
						return (256 & this._lView[2]) == 256;
					}
					destroy() {
						if (this._appRef) this._appRef.detachView(this);
						else if (this._attachedToViewContainer) {
							let e = this._lView[3];
							if (ts(e)) {
								let t = e[8],
									n = t ? t.indexOf(this) : -1;
								n > -1 &&
									(ngDevMode &&
										V(
											n,
											e.indexOf(this._lView) - 11,
											"An attached view should be in the same position within its container as its ViewRef in the VIEW_REFS array.",
										),
									oO(e, n),
									rC(t, n));
							}
							this._attachedToViewContainer = !1;
						}
						oA(this._lView[1], this._lView);
					}
					onDestroy(e) {
						t2(this._lView, e);
					}
					markForCheck() {
						sP(this._cdRefInjectingView || this._lView);
					}
					detach() {
						this._lView[2] &= -129;
					}
					reattach() {
						this._lView[2] |= 128;
					}
					detectChanges() {
						lR(this._lView[1], this._lView, this.context);
					}
					checkNoChanges() {
						ngDevMode && lN(this._lView[1], this._lView, this.context);
					}
					attachToViewContainerRef() {
						if (this._appRef)
							throw new O(
								902,
								ngDevMode &&
									"This view is already attached directly to the ApplicationRef!",
							);
						this._attachedToViewContainer = !0;
					}
					detachFromAppRef() {
						var e, t;
						(this._appRef = null),
							(e = this._lView[1]),
							oG(e, (t = this._lView), t[11], 2, null, null);
					}
					attachToAppRef(e) {
						if (this._attachedToViewContainer)
							throw new O(
								902,
								ngDevMode &&
									"This view is already attached to a ViewContainer!",
							);
						this._appRef = e;
					}
					constructor(e, t) {
						(this._lView = e),
							(this._cdRefInjectingView = t),
							(this._appRef = null),
							(this._attachedToViewContainer = !1);
					}
				}
				class lV extends l$ {
					detectChanges() {
						let e = this._view,
							t = e[1],
							n = e[8];
						lR(t, e, n, !1);
					}
					checkNoChanges() {
						if (ngDevMode) {
							let e = this._view,
								t = e[1],
								n = e[8];
							lN(t, e, n, !1);
						}
					}
					get context() {
						return null;
					}
					constructor(e) {
						super(e), (this._view = e);
					}
				}
				class lB extends sj {
					resolveComponentFactory(e) {
						ngDevMode &&
							(function (
								e,
								t = "Type passed in is not ComponentType, it does not have 'ɵcmp' property.",
							) {
								!e6(e) && Y(t);
							})(e);
						let t = e6(e);
						return new lz(t, this.ngModule);
					}
					constructor(e) {
						super(), (this.ngModule = e);
					}
				}
				function lU(e) {
					let t = [];
					for (let n in e)
						if (e.hasOwnProperty(n)) {
							let r = e[n];
							t.push({ propName: r, templateName: n });
						}
					return t;
				}
				class lH {
					get(e, t, n) {
						n = eD(n);
						let r = this.injector.get(e, sI, n);
						return r !== sI || t === sI ? r : this.parentInjector.get(e, t, n);
					}
					constructor(e, t) {
						(this.injector = e), (this.parentInjector = t);
					}
				}
				class lz extends sy {
					get inputs() {
						return lU(this.componentDef.inputs);
					}
					get outputs() {
						return lU(this.componentDef.outputs);
					}
					create(e, t, n, r) {
						let o, i;
						let s =
							(r = r || this.ngModule) instanceof st
								? r
								: null == r
								? void 0
								: r.injector;
						s &&
							null !== this.componentDef.getStandaloneInjector &&
							(s = this.componentDef.getStandaloneInjector(s) || s);
						let l = s ? new lH(e, s) : e,
							a = l.get(sC, null);
						if (null === a)
							throw new O(
								407,
								ngDevMode &&
									"Angular was not able to inject a renderer (RendererFactory2). Likely this is due to a broken DI hierarchy. Make sure that any injector used to create this component has a correct parent.",
							);
						let u = l.get(sE, null),
							d = l.get(lk, null),
							c = { rendererFactory: a, sanitizer: u, effectManager: d },
							f = a.createRenderer(null, this.componentDef),
							h = this.componentDef.selectors[0][0] || "div",
							p = n
								? (function (e, t, n, r) {
										let o = r.get(sF, !1),
											i = o || n === eE.ShadowDom,
											s = e.selectRootElement(t, i);
										return (
											(function (e) {
												ll(e);
											})(s),
											s
										);
								  })(f, n, this.componentDef.encapsulation, l)
								: oS(
										f,
										h,
										(function (e) {
											let t = e.toLowerCase();
											return "svg" === t ? "svg" : "math" === t ? tz : null;
										})(h),
								  ),
							m = this.componentDef.onPush ? 576 : 528,
							g = ls(0, null, null, 1, 0, null, null, null, null, null, null),
							v = s8(null, g, null, m, null, null, c, f, l, null, null);
						nx(v);
						try {
							let e;
							let r = this.componentDef,
								s = null;
							r.findHostDirectiveDefs
								? ((e = []),
								  (s = new Map()),
								  r.findHostDirectiveDefs(r, e, s),
								  e.push(r))
								: (e = [r]);
							let l = (function (e, t) {
									let n = e[1];
									return (
										ngDevMode && K(e, 25),
										(e[25] = t),
										s7(n, 25, 2, "#host", null)
									);
								})(v, p),
								a = (function (e, t, n, r, o, i, s) {
									let l = o[1];
									(function (e, t, n, r) {
										for (let n of e)
											t.mergedAttrs = eU(t.mergedAttrs, n.hostAttrs);
										null !== t.mergedAttrs &&
											(lF(t, t.mergedAttrs, !0), null !== n && oQ(r, n, t));
									})(r, e, t, s);
									let a = null;
									if (null !== t) a = sm(t, o[9]);
									let u = i.rendererFactory.createRenderer(t, n),
										d = s8(
											o,
											li(n),
											null,
											n.onPush ? 64 : 16,
											o[e.index],
											e,
											i,
											u,
											null,
											null,
											a,
										);
									return (
										l.firstCreatePass && lm(l, e, r.length - 1),
										lj(o, d),
										(o[e.index] = d)
									);
								})(l, p, r, e, v, c, f);
							(i = tZ(g, 25)),
								p &&
									(function (e, t, n, r) {
										if (r) e$(e, n, ["ng-version", sA.full]);
										else {
											let { attrs: r, classes: o } = (function (e) {
												let t = [],
													n = [],
													r = 1,
													o = 2;
												for (; r < e.length; ) {
													let i = e[r];
													if ("string" == typeof i)
														2 === o
															? "" !== i && t.push(i, e[++r])
															: 8 === o && n.push(i);
													else {
														if (!eq(o)) break;
														o = i;
													}
													r++;
												}
												return { attrs: t, classes: n };
											})(t.selectors[0]);
											r && e$(e, n, r),
												o && o.length > 0 && oY(e, n, o.join(" "));
										}
									})(f, r, p, n),
								void 0 !== t &&
									(function (e, t, n) {
										let r = (e.projection = []);
										for (let e = 0; e < t.length; e++) {
											let t = n[e];
											r.push(null != t ? Array.from(t) : null);
										}
									})(i, this.ngContentSelectors, t),
								(o = (function (e, t, n, r, o, i) {
									let s = nr();
									ngDevMode && Z(s, "tNode should have been already created");
									let l = o[1],
										a = tG(s, o);
									lh(l, o, s, n, null, r);
									for (let e = 0; e < n.length; e++) {
										let t = s.directiveStart + e,
											n = n9(o, l, t, s);
										oc(n, o);
									}
									lp(l, o, s),
										a && oc(a, o),
										ngDevMode &&
											q(
												s.componentOffset,
												-1,
												"componentOffset must be great than -1",
											);
									let u = n9(o, l, s.directiveStart + s.componentOffset, s);
									if (((e[8] = o[8] = u), null !== i)) for (let e of i) e(u, t);
									return ln(l, s, e), u;
								})(a, r, e, s, v, [lq])),
								lA(g, v, null);
						} finally {
							nC();
						}
						return new lW(this.componentType, o, sD(i, v), v, i);
					}
					constructor(e, t) {
						super(),
							(this.componentDef = e),
							(this.ngModule = t),
							(this.componentType = e.type),
							(this.selector = eQ(e.selectors)),
							(this.ngContentSelectors = e.ngContentSelectors
								? e.ngContentSelectors
								: []),
							(this.isBoundToModule = !!t);
					}
				}
				class lW extends sv {
					setInput(e, t) {
						let n;
						let r = this._tNode.inputs;
						if (null !== r && (n = r[e])) {
							var o;
							if (
								((null !== (o = this.previousInputValues) && void 0 !== o) ||
									(this.previousInputValues = new Map()),
								this.previousInputValues.has(e) &&
									Object.is(this.previousInputValues.get(e), t))
							)
								return;
							let r = this._rootLView;
							lE(r[1], r, n, e, t), this.previousInputValues.set(e, t);
							let i = tQ(this._tNode.index, r);
							sP(i);
						} else if (ngDevMode) {
							let t = P(this.componentType),
								n = `Can't set value of the '${e}' input on the '${t}' component. `;
							r2(
								(n += `Make sure that the '${e}' property is annotated with @Input() or a mapped @Input('${e}') exists.`),
							);
						}
					}
					get injector() {
						return new rn(this._tNode, this._rootLView);
					}
					destroy() {
						this.hostView.destroy();
					}
					onDestroy(e) {
						this.hostView.onDestroy(e);
					}
					constructor(e, t, n, r, o) {
						super(),
							(this.location = n),
							(this._rootLView = r),
							(this._tNode = o),
							(this.previousInputValues = null),
							(this.instance = t),
							(this.hostView = this.changeDetectorRef = new lV(r)),
							(this.componentType = e);
					}
				}
				function lq() {
					let e = nr();
					ngDevMode && Z(e, "TNode is required"), nR(t9()[1], e);
				}
				function lG(e) {
					return Object.getPrototypeOf(e.prototype).constructor;
				}
				function lZ(e) {
					let t = lG(e.type),
						n = !0,
						r = [e];
					for (; t; ) {
						let o;
						if (td(e)) o = t.ɵcmp || t.ɵdir;
						else {
							if (t.ɵcmp)
								throw new O(
									903,
									ngDevMode &&
										`Directives cannot inherit Components. Directive ${P(
											e.type,
										)} is attempting to extend component ${P(t)}`,
								);
							o = t.ɵdir;
						}
						if (o) {
							if (n) {
								r.push(o);
								(e.inputs = lY(e.inputs)),
									(e.declaredInputs = lY(e.declaredInputs)),
									(e.outputs = lY(e.outputs));
								let t = o.hostBindings;
								t &&
									(function (e, t) {
										let n = e.hostBindings;
										n
											? (e.hostBindings = (e, r) => {
													t(e, r), n(e, r);
											  })
											: (e.hostBindings = t);
									})(e, t);
								let n = o.viewQuery,
									i = o.contentQueries;
								if (
									(n &&
										(function (e, t) {
											let n = e.viewQuery;
											n
												? (e.viewQuery = (e, r) => {
														t(e, r), n(e, r);
												  })
												: (e.viewQuery = t);
										})(e, n),
									i &&
										(function (e, t) {
											let n = e.contentQueries;
											n
												? (e.contentQueries = (e, r, o) => {
														t(e, r, o), n(e, r, o);
												  })
												: (e.contentQueries = t);
										})(e, i),
									_(e.inputs, o.inputs),
									_(e.declaredInputs, o.declaredInputs),
									_(e.outputs, o.outputs),
									td(o) && o.data.animation)
								) {
									let t = e.data;
									t.animation = (t.animation || []).concat(o.data.animation);
								}
							}
							let t = o.features;
							if (t)
								for (let r = 0; r < t.length; r++) {
									let o = t[r];
									o && o.ngInherit && o(e), o === lZ && (n = !1);
								}
						}
						t = Object.getPrototypeOf(t);
					}
					(function (e) {
						let t = 0,
							n = null;
						for (let r = e.length - 1; r >= 0; r--) {
							let o = e[r];
							(o.hostVars = t += o.hostVars),
								(o.hostAttrs = eU(o.hostAttrs, (n = eU(n, o.hostAttrs))));
						}
					})(r);
				}
				function lY(e) {
					return e === eO ? {} : e === eA ? [] : e;
				}
				let lQ = ["providersResolver"],
					lK = [
						"template",
						"decls",
						"consts",
						"vars",
						"onPush",
						"ngContentSelectors",
						"styles",
						"encapsulation",
						"schemas",
					];
				function lJ(e) {
					let t,
						n = lG(e.type);
					t = td(e) ? n.ɵcmp : n.ɵdir;
					for (let n of lQ) e[n] = t[n];
					if (td(t)) for (let n of lK) e[n] = t[n];
				}
				function lX(e) {
					return (t) => {
						(t.findHostDirectiveDefs = function e(t, n, r) {
							if (null !== t.hostDirectives)
								for (let o of t.hostDirectives) {
									let t = e8(o.directive);
									("undefined" == typeof ngDevMode || ngDevMode) &&
										(function (e, t, n) {
											let r = e.directive;
											if (null === t) {
												if (null !== e6(r))
													throw new O(
														310,
														`Host directive ${r.name} cannot be a component.`,
													);
												throw new O(
													307,
													`Could not resolve metadata for host directive ${r.name}. Make sure that the ${r.name} class is annotated with an @Directive decorator.`,
												);
											}
											if (!t.standalone)
												throw new O(
													308,
													`Host directive ${t.type.name} must be standalone.`,
												);
											if (n.indexOf(t) > -1)
												throw new O(
													309,
													`Directive ${t.type.name} matches multiple times on the same element. Directives can only match an element once.`,
												);
											l1("input", t, e.inputs), l1("output", t, e.outputs);
										})(o, t, n),
										(function (e, t) {
											for (let n in t)
												if (t.hasOwnProperty(n)) {
													let r = t[n],
														o = e[n];
													("undefined" == typeof ngDevMode || ngDevMode) &&
														e.hasOwnProperty(r) &&
														V(
															e[r],
															e[n],
															`Conflicting host directive input alias ${n}.`,
														),
														(e[r] = o);
												}
										})(t.declaredInputs, o.inputs),
										e(t, n, r),
										r.set(t, o),
										n.push(t);
								}
						}),
							(t.hostDirectives = (Array.isArray(e) ? e : e()).map((e) =>
								"function" == typeof e
									? { directive: M(e), inputs: eO, outputs: eO }
									: {
											directive: M(e.directive),
											inputs: l0(e.inputs),
											outputs: l0(e.outputs),
									  },
							));
					};
				}
				function l0(e) {
					if (void 0 === e || 0 === e.length) return eO;
					let t = {};
					for (let n = 0; n < e.length; n += 2) t[e[n]] = e[n + 1];
					return t;
				}
				function l1(e, t, n) {
					let r = t.type.name,
						o = "input" === e ? t.inputs : t.outputs;
					for (let t in n)
						if (n.hasOwnProperty(t)) {
							if (!o.hasOwnProperty(t))
								throw new O(
									311,
									`Directive ${r} does not have an ${e} with a public name of ${t}.`,
								);
							let i = n[t];
							if (o.hasOwnProperty(i) && o[i] !== t)
								throw new O(
									312,
									`Cannot alias ${e} ${t} of host directive ${r} to ${i}, because it already has a different ${e} with the same public name.`,
								);
						}
				}
				function l5(e) {
					return (
						!!l2(e) &&
						(Array.isArray(e) || (!(e instanceof Map) && Symbol.iterator in e))
					);
				}
				function l2(e) {
					return null !== e && ("function" == typeof e || "object" == typeof e);
				}
				function l3(e, t, n) {
					return (e[t] = n);
				}
				function l4(e, t) {
					return (
						ngDevMode && K(e, t),
						ngDevMode && H(e[t], sZ, "Stored value should never be NO_CHANGE."),
						e[t]
					);
				}
				function l6(e, t, n) {
					ngDevMode && H(n, sZ, "Incoming value should never be NO_CHANGE."),
						ngDevMode &&
							z(t, e.length, "Slot should have been initialized to NO_CHANGE");
					let r = e[t];
					if (Object.is(r, n)) return !1;
					if (ngDevMode && nu()) {
						let o = r !== sZ ? r : void 0;
						if (
							!(function e(t, n) {
								let r = l5(t),
									o = l5(n);
								if (r && o)
									return (function (e, t, n) {
										let r = e[Symbol.iterator](),
											o = t[Symbol.iterator]();
										for (;;) {
											let e = r.next(),
												t = o.next();
											if (e.done && t.done) return !0;
											if (e.done || t.done || !n(e.value, t.value)) return !1;
										}
									})(t, n, e);
								{
									let e = t && ("object" == typeof t || "function" == typeof t),
										i = n && ("object" == typeof n || "function" == typeof n);
									return (!r && !!e && !o && !!i) || Object.is(t, n);
								}
							})(o, n)
						) {
							let i = (function (e, t, n, r) {
								let o = e[1].data,
									i = o[t];
								if ("string" == typeof i)
									return i.indexOf(sV) > -1
										? sU(e, t, t, i, r)
										: { propName: i, oldValue: n, newValue: r };
								if (null === i) {
									let n = t - 1;
									for (; "string" != typeof o[n] && null === o[n + 1]; ) n--;
									let i = o[n];
									if ("string" == typeof i) {
										let o = i.match(RegExp(sV, "g"));
										if (o && o.length - 1 > t - n) return sU(e, n, t, i, r);
									}
								}
								return { propName: void 0, oldValue: n, newValue: r };
							})(e, t, o, n);
							!(function (e, t, n, r) {
								let o = r ? ` for '${r}'` : "",
									i = `ExpressionChangedAfterItHasBeenCheckedError: Expression has changed after it was checked. Previous value${o}: '${t}'. Current value: '${n}'.`;
								throw (
									(e &&
										(i +=
											" It seems like the view has been created after its parent and its children have been dirty checked. Has it been created in a change detection hook?"),
									new O(-100, i))
								);
							})(r === sZ, i.oldValue, i.newValue, i.propName);
						}
						return !1;
					}
					return (e[t] = n), !0;
				}
				function l8(e, t, n, r) {
					let o = l6(e, t, n);
					return l6(e, t + 1, r) || o;
				}
				function l7(e, t, n, r, o) {
					let i = l8(e, t, n, r);
					return l6(e, t + 2, o) || i;
				}
				function l9(e, t, n, r, o, i) {
					let s = l8(e, t, n, r);
					return l8(e, t + 2, o, i) || s;
				}
				function ae(e, t, n, r) {
					let o = t9(),
						i = np();
					if (l6(o, i, t)) {
						let s = ne(),
							l = nO();
						lg(l, o, e, t, n, r), ngDevMode && lD(s.data, l, "attr." + e, i);
					}
					return ae;
				}
				function at(e, t) {
					ngDevMode && z(2, t.length, "should have at least 3 values"),
						ngDevMode &&
							V(t.length % 2, 1, "should have an odd number of values");
					let n = !1,
						r = nf();
					for (let o = 1; o < t.length; o += 2) n = l6(e, r++, t[o]) || n;
					if ((nh(r), !n)) return sZ;
					let o = t[0];
					for (let e = 1; e < t.length; e += 2) o += I(t[e]) + t[e + 1];
					return o;
				}
				function an(e, t, n, r) {
					let o = l6(e, np(), n);
					return o ? t + I(n) + r : sZ;
				}
				function ar(e, t, n, r, o, i) {
					let s = nf(),
						l = l8(e, s, n, o);
					return nm(2), l ? t + I(n) + r + I(o) + i : sZ;
				}
				function ao(e, t, n, r, o, i, s, l) {
					let a = nf(),
						u = l7(e, a, n, o, s);
					return nm(3), u ? t + I(n) + r + I(o) + i + I(s) + l : sZ;
				}
				function ai(e, t, n, r, o, i, s, l, a, u) {
					let d = nf(),
						c = l9(e, d, n, o, s, a);
					return nm(4), c ? t + I(n) + r + I(o) + i + I(s) + l + I(a) + u : sZ;
				}
				function as(e, t, n, r, o, i, s, l, a, u, d, c) {
					let f = nf(),
						h = l9(e, f, n, o, s, a);
					return (
						(h = l6(e, f + 4, d) || h),
						nm(5),
						h ? t + I(n) + r + I(o) + i + I(s) + l + I(a) + u + I(d) + c : sZ
					);
				}
				function al(e, t, n, r, o, i, s, l, a, u, d, c, f, h) {
					let p = nf(),
						m = l9(e, p, n, o, s, a);
					return (
						(m = l8(e, p + 4, d, f) || m),
						nm(6),
						m
							? t +
							  I(n) +
							  r +
							  I(o) +
							  i +
							  I(s) +
							  l +
							  I(a) +
							  u +
							  I(d) +
							  c +
							  I(f) +
							  h
							: sZ
					);
				}
				function aa(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m) {
					let g = nf(),
						v = l9(e, g, n, o, s, a);
					return (
						(v = l7(e, g + 4, d, f, p) || v),
						nm(7),
						v
							? t +
							  I(n) +
							  r +
							  I(o) +
							  i +
							  I(s) +
							  l +
							  I(a) +
							  u +
							  I(d) +
							  c +
							  I(f) +
							  h +
							  I(p) +
							  m
							: sZ
					);
				}
				function au(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v) {
					let y = nf(),
						b = l9(e, y, n, o, s, a);
					return (
						(b = l9(e, y + 4, d, f, p, g) || b),
						nm(8),
						b
							? t +
							  I(n) +
							  r +
							  I(o) +
							  i +
							  I(s) +
							  l +
							  I(a) +
							  u +
							  I(d) +
							  c +
							  I(f) +
							  h +
							  I(p) +
							  m +
							  I(g) +
							  v
							: sZ
					);
				}
				function ad(e, t, n, r, o, i) {
					let s = t9(),
						l = an(s, t, n, r);
					if (l !== sZ) {
						let n = nO();
						lg(n, s, e, l, o, i),
							ngDevMode && lD(ne().data, n, "attr." + e, nf() - 1, t, r);
					}
					return ad;
				}
				function ac(e, t, n, r, o, i, s, l) {
					let a = t9(),
						u = ar(a, t, n, r, o, i);
					if (u !== sZ) {
						let n = nO();
						lg(n, a, e, u, s, l),
							ngDevMode && lD(ne().data, n, "attr." + e, nf() - 2, t, r, i);
					}
					return ac;
				}
				function af(e, t, n, r, o, i, s, l, a, u) {
					let d = t9(),
						c = ao(d, t, n, r, o, i, s, l);
					if (c !== sZ) {
						let n = nO();
						lg(n, d, e, c, a, u),
							ngDevMode && lD(ne().data, n, "attr." + e, nf() - 3, t, r, i, l);
					}
					return af;
				}
				function ah(e, t, n, r, o, i, s, l, a, u, d, c) {
					let f = t9(),
						h = ai(f, t, n, r, o, i, s, l, a, u);
					if (h !== sZ) {
						let n = nO();
						lg(n, f, e, h, d, c),
							ngDevMode &&
								lD(ne().data, n, "attr." + e, nf() - 4, t, r, i, l, u);
					}
					return ah;
				}
				function ap(e, t, n, r, o, i, s, l, a, u, d, c, f, h) {
					let p = t9(),
						m = as(p, t, n, r, o, i, s, l, a, u, d, c);
					if (m !== sZ) {
						let n = nO();
						lg(n, p, e, m, f, h),
							ngDevMode &&
								lD(ne().data, n, "attr." + e, nf() - 5, t, r, i, l, u, c);
					}
					return ap;
				}
				function am(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m) {
					let g = t9(),
						v = al(g, t, n, r, o, i, s, l, a, u, d, c, f, h);
					if (v !== sZ) {
						let n = nO();
						lg(n, g, e, v, p, m),
							ngDevMode &&
								lD(ne().data, n, "attr." + e, nf() - 6, t, r, i, l, u, c, h);
					}
					return am;
				}
				function ag(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v) {
					let y = t9(),
						b = aa(y, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m);
					if (b !== sZ) {
						let n = nO();
						lg(n, y, e, b, g, v),
							ngDevMode &&
								lD(ne().data, n, "attr." + e, nf() - 7, t, r, i, l, u, c, h, m);
					}
					return ag;
				}
				function av(
					e,
					t,
					n,
					r,
					o,
					i,
					s,
					l,
					a,
					u,
					d,
					c,
					f,
					h,
					p,
					m,
					g,
					v,
					y,
					b,
				) {
					let _ = t9(),
						j = au(_, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v);
					if (j !== sZ) {
						let n = nO();
						lg(n, _, e, j, y, b),
							ngDevMode &&
								lD(
									ne().data,
									n,
									"attr." + e,
									nf() - 8,
									t,
									r,
									i,
									l,
									u,
									c,
									h,
									m,
									v,
								);
					}
					return av;
				}
				function ay(e, t, n, r) {
					let o = t9(),
						i = at(o, t);
					if (i !== sZ) {
						let s = nO();
						if ((lg(s, o, e, i, n, r), ngDevMode)) {
							let n = [t[0]];
							for (let e = 2; e < t.length; e += 2) n.push(t[e]);
							lD(ne().data, s, "attr." + e, nf() - n.length + 1, ...n);
						}
					}
					return ay;
				}
				RegExp("^(\\d+)*(b|h)*(.*)");
				function ab(e, t, n, r, o, i, s, l) {
					let a = t9(),
						u = ne(),
						d = e + 25,
						c = u.firstCreatePass
							? (function (e, t, n, r, o, i, s, l, a) {
									ngDevMode && tb(t), ngDevMode && ngDevMode.firstCreatePass++;
									let u = t.consts,
										d = s7(t, e, 4, s || null, tX(u, l));
									lf(t, n, d, tX(u, a)), nR(t, d);
									let c = (d.tView = ls(
										2,
										d,
										r,
										o,
										i,
										t.directiveRegistry,
										t.pipeRegistry,
										null,
										t.schemas,
										u,
										null,
									));
									return (
										null !== t.queries &&
											(t.queries.template(t, d),
											(c.queries = t.queries.embeddedTView(d))),
										d
									);
							  })(d, u, a, t, n, r, o, i, s)
							: u.data[d];
					ns(c, !1);
					let f = a_(u, a, c, e);
					nT && oU(u, a, f, c),
						oc(f, a),
						lj(a, (a[d] = lb(f, a, f, c))),
						tu(c) && lr(u, a, c),
						null != s && lo(a, c, l);
				}
				let a_ = function (e, t, n, r) {
					return (nT = !0), t[11].createComment(ngDevMode ? "container" : "");
				};
				function aj(e) {
					let t = (function () {
						let e = t3.lFrame.contextLView;
						return ngDevMode && Z(e, "contextLView must be defined."), e;
					})();
					return tY(t, 25 + e);
				}
				function ax(e, t, n) {
					let r = t9(),
						o = np();
					if (l6(r, o, t)) {
						let i = ne(),
							s = nO();
						ld(i, s, r, e, t, r[11], n, !1), ngDevMode && lD(i.data, s, e, o);
					}
					return ax;
				}
				function aD(e, t, n, r, o) {
					let i = t.inputs,
						s = o ? "class" : "style";
					lE(e, n, i[s], s, r);
				}
				function aw(e, t, n, r) {
					let o = t9(),
						i = ne(),
						s = 25 + e;
					ngDevMode &&
						V(
							nf(),
							i.bindingStartIndex,
							"elements should be created before any bindings",
						),
						ngDevMode && K(o, s);
					let l = o[11],
						a = i.firstCreatePass
							? (function (e, t, n, r, o, i) {
									ngDevMode && tb(t), ngDevMode && ngDevMode.firstCreatePass++;
									let s = t.consts,
										l = tX(s, o),
										a = s7(t, e, 2, r, l);
									return (
										lf(t, n, a, tX(s, i)),
										null !== a.attrs && lF(a, a.attrs, !1),
										null !== a.mergedAttrs && lF(a, a.mergedAttrs, !0),
										null !== t.queries && t.queries.elementStart(t, a),
										a
									);
							  })(s, i, o, t, n, r)
							: i.data[s],
						u = aS(i, o, a, l, t, e);
					o[s] = u;
					let d = tu(a);
					return (
						ngDevMode &&
							i.firstCreatePass &&
							!(function (e, t, n, r, o) {
								if (null !== r && !o && null !== n) {
									let o =
										("undefined" != typeof HTMLUnknownElement &&
											HTMLUnknownElement &&
											e instanceof HTMLUnknownElement) ||
										("undefined" != typeof customElements &&
											n.indexOf("-") > -1 &&
											!customElements.get(n));
									if (o && !r7(r, n)) {
										let e = r4(t),
											r = r6(t),
											o = `'${e ? "@Component" : "@NgModule"}.schemas'`,
											i = `'${n}' is not a known element${r}:
`;
										(i += `1. If '${n}' is an Angular component, then verify that it is ${
											e
												? "included in the '@Component.imports' of this component"
												: "a part of an @NgModule where this component is declared"
										}.
`),
											n && n.indexOf("-") > -1
												? (i += `2. If '${n}' is a Web Component then add 'CUSTOM_ELEMENTS_SCHEMA' to the ${o} of this component to suppress this message.`)
												: (i += `2. To allow any element add 'NO_ERRORS_SCHEMA' to the ${o} of this component.`);
										console.error(A(304, i));
									}
								}
							})(u, o, a.value, i.schemas, d),
						ns(a, !0),
						oQ(l, u, a),
						(32 & a.flags) != 32 && nT && oU(i, o, u, a),
						0 === t3.lFrame.elementDepthCount && oc(u, o),
						t3.lFrame.elementDepthCount++,
						d && (lr(i, o, a), ln(i, a, o)),
						null !== r && lo(o, a),
						aw
					);
				}
				function aM() {
					var e;
					let t = nr();
					ngDevMode && Z(t, "No parent node to close."),
						nl() ? na() : (ngDevMode && tm(nr()), ns((t = t.parent), !1));
					let n = t;
					if ((ngDevMode && nz(n, 3), (e = n), t3.skipHydrationRootTNode === e))
						t3.skipHydrationRootTNode = null;
					t3.lFrame.elementDepthCount--;
					let r = ne();
					if (
						(r.firstCreatePass && (nR(r, t), tl(t) && r.queries.elementEnd(t)),
						null != n.classesWithoutHost && (8 & n.flags) != 0)
					)
						aD(r, n, t9(), n.classesWithoutHost, !0);
					if (null != n.stylesWithoutHost && (16 & n.flags) != 0)
						aD(r, n, t9(), n.stylesWithoutHost, !1);
					return aM;
				}
				function aC(e, t, n, r) {
					return aw(e, t, n, r), aM(), aC;
				}
				let aS = (e, t, n, r, o, i) => (
					(nT = !0), oS(r, o, t3.lFrame.currentNamespace)
				);
				function aE(e, t, n) {
					let r = t9(),
						o = ne(),
						i = e + 25;
					ngDevMode && K(r, i),
						ngDevMode &&
							V(
								nf(),
								o.bindingStartIndex,
								"element containers should be created before any bindings",
							);
					let s = o.firstCreatePass
						? (function (e, t, n, r, o) {
								ngDevMode && ngDevMode.firstCreatePass++;
								let i = t.consts,
									s = tX(i, r),
									l = s7(t, e, 8, "ng-container", s);
								null !== s && lF(l, s, !0);
								let a = tX(i, o);
								return (
									lf(t, n, l, a),
									null !== t.queries && t.queries.elementStart(t, l),
									l
								);
						  })(i, o, r, t, n)
						: o.data[i];
					ns(s, !0);
					let l = aI(o, r, s, e);
					return (
						(r[i] = l),
						nT && oU(o, r, l, s),
						oc(l, r),
						tu(s) && (lr(o, r, s), ln(o, s, r)),
						null != n && lo(r, s),
						aE
					);
				}
				function aO() {
					let e = nr(),
						t = ne();
					return (
						nl() ? na() : (ngDevMode && tm(e), ns((e = e.parent), !1)),
						ngDevMode && nz(e, 8),
						t.firstCreatePass && (nR(t, e), tl(e) && t.queries.elementEnd(e)),
						aO
					);
				}
				function aA(e, t, n) {
					return aE(e, t, n), aO(), aA;
				}
				let aI = (e, t, n, r) => (
					(nT = !0), oC(t[11], ngDevMode ? "ng-container" : "")
				);
				function aP() {
					return t9();
				}
				function aT(e) {
					return !!e && "function" == typeof e.then;
				}
				function ak(e) {
					return !!e && "function" == typeof e.subscribe;
				}
				function aF(e, t, n, r) {
					let o = t9(),
						i = ne(),
						s = nr();
					return aN(i, o, o[11], s, e, t, r), aF;
				}
				function aR(e, t) {
					let n = nr(),
						r = t9(),
						o = ne(),
						i = ny(o.data),
						s = lC(i, n, r);
					return aN(o, r, s, n, e, t), aR;
				}
				function aN(e, t, n, r, o, i, s) {
					let l;
					let a = tu(r),
						u = e.firstCreatePass,
						d = u && lM(e),
						c = t[8],
						f = lw(t);
					ngDevMode && nz(r, 15);
					let h = !0;
					if (3 & r.type || s) {
						let l = tG(r, t),
							u = s ? s(l) : l,
							p = f.length,
							m = s ? (e) => s(tW(e[r.index])) : r.index,
							g = null;
						if (
							(!s &&
								a &&
								(g = (function (e, t, n, r) {
									let o = e.cleanup;
									if (null != o)
										for (let e = 0; e < o.length - 1; e += 2) {
											let i = o[e];
											if (i === n && o[e + 1] === r) {
												let n = t[7],
													r = o[e + 2];
												return n.length > r ? n[r] : null;
											}
											"string" == typeof i && (e += 2);
										}
									return null;
								})(e, t, o, r.index)),
							null !== g)
						) {
							let e = g.__ngLastListenerFn__ || g;
							(e.__ngNextListenerFn__ = i),
								(g.__ngLastListenerFn__ = i),
								(h = !1);
						} else {
							i = a$(r, t, c, i, !1);
							let e = n.listen(u, o, i);
							ngDevMode && ngDevMode.rendererAddEventListener++,
								f.push(i, e),
								d && d.push(o, m, p, p + 1);
						}
					} else i = a$(r, t, c, i, !1);
					let p = r.outputs;
					if (h && null !== p && (l = p[o])) {
						let e = l.length;
						if (e)
							for (let n = 0; n < e; n += 2) {
								let e = l[n];
								ngDevMode && K(t, e);
								let s = l[n + 1],
									a = t[e],
									u = a[s];
								if (ngDevMode && !ak(u))
									throw Error(
										`@Output ${s} not initialized in '${a.constructor.name}'.`,
									);
								let c = u.subscribe(i),
									h = f.length;
								f.push(i, c), d && d.push(o, r.index, h, -(h + 1));
							}
					}
				}
				function aL(e, t, n, r) {
					try {
						return tH(6, t, n), !1 !== n(r);
					} catch (t) {
						return lS(e, t), !1;
					} finally {
						tH(7, t, n);
					}
				}
				function a$(e, t, n, r, o) {
					return function i(s) {
						if (s === Function) return r;
						let l = e.componentOffset > -1 ? tQ(e.index, t) : t;
						sP(l);
						let a = aL(t, n, r, s),
							u = i.__ngNextListenerFn__;
						for (; u; ) (a = aL(t, n, u, s) && a), (u = u.__ngNextListenerFn__);
						return (
							o && !1 === a && (s.preventDefault(), (s.returnValue = !1)), a
						);
					};
				}
				function aV(e = 1) {
					return (function (e) {
						let t = (t3.lFrame.contextLView = (function (e, t) {
							for (; e > 0; )
								ngDevMode &&
									Z(
										t[14],
										"Declaration view should be defined if nesting level is greater than 0.",
									),
									(t = t[14]),
									e--;
							return t;
						})(e, t3.lFrame.contextLView));
						return t[8];
					})(e);
				}
				function aB(e) {
					let t = t9()[15][6];
					if (!t.projection) {
						let n = e ? e.length : 1,
							r = (t.projection = rS(n, null)),
							o = r.slice(),
							i = t.child;
						for (; null !== i; ) {
							let t = e
								? (function (e, t) {
										let n = null,
											r = (function (e) {
												let t = e.attrs;
												if (null != t) {
													let e = t.indexOf(5);
													if ((1 & e) == 0) return t[e + 1];
												}
												return null;
											})(e);
										for (let o = 0; o < t.length; o++) {
											let i = t[o];
											if ("*" === i) {
												n = o;
												continue;
											}
											if (
												null === r
													? eG(e, i, !0)
													: (function (e, t) {
															e: for (let n = 0; n < t.length; n++) {
																let r = t[n];
																if (e.length === r.length) {
																	for (let t = 0; t < e.length; t++)
																		if (e[t] !== r[t]) continue e;
																	return !0;
																}
															}
															return !1;
													  })(r, i)
											)
												return o;
										}
										return n;
								  })(i, e)
								: 0;
							null !== t &&
								(o[t] ? (o[t].projectionNext = i) : (r[t] = i), (o[t] = i)),
								(i = i.next);
						}
					}
				}
				function aU(e, t = 0, n) {
					let r = t9(),
						o = ne(),
						i = s7(o, 25 + e, 16, null, n || null);
					null === i.projection && (i.projection = t), na();
					let s = r[22],
						l = !s || null !== t3.skipHydrationRootTNode;
					l &&
						(32 & i.flags) != 32 &&
						!(function (e, t, n) {
							var r, o, i;
							let s = t[11];
							let l = ((r = e), (o = n), (i = t), oT(r, o.parent, i)),
								a = n.parent || t[6],
								u = oV(a, n, t);
							oZ(s, 0, t, n, l, u);
						})(o, r, i);
				}
				function aH(e, t, n) {
					return az(e, "", t, "", n), aH;
				}
				function az(e, t, n, r, o) {
					let i = t9(),
						s = an(i, t, n, r);
					if (s !== sZ) {
						let n = ne(),
							l = nO();
						ld(n, l, i, e, s, i[11], o, !1),
							ngDevMode && lD(n.data, l, e, nf() - 1, t, r);
					}
					return az;
				}
				function aW(e, t, n, r, o, i, s) {
					let l = t9(),
						a = ar(l, t, n, r, o, i);
					if (a !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, l, e, a, l[11], s, !1),
							ngDevMode && lD(n.data, o, e, nf() - 2, t, r, i);
					}
					return aW;
				}
				function aq(e, t, n, r, o, i, s, l, a) {
					let u = t9(),
						d = ao(u, t, n, r, o, i, s, l);
					if (d !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, u, e, d, u[11], a, !1),
							ngDevMode && lD(n.data, o, e, nf() - 3, t, r, i, l);
					}
					return aq;
				}
				function aG(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t9(),
						f = ai(c, t, n, r, o, i, s, l, a, u);
					if (f !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, c, e, f, c[11], d, !1),
							ngDevMode && lD(n.data, o, e, nf() - 4, t, r, i, l, u);
					}
					return aG;
				}
				function aZ(e, t, n, r, o, i, s, l, a, u, d, c, f) {
					let h = t9(),
						p = as(h, t, n, r, o, i, s, l, a, u, d, c);
					if (p !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, h, e, p, h[11], f, !1),
							ngDevMode && lD(n.data, o, e, nf() - 5, t, r, i, l, u, c);
					}
					return aZ;
				}
				function aY(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p) {
					let m = t9(),
						g = al(m, t, n, r, o, i, s, l, a, u, d, c, f, h);
					if (g !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, m, e, g, m[11], p, !1),
							ngDevMode && lD(n.data, o, e, nf() - 6, t, r, i, l, u, c, h);
					}
					return aY;
				}
				function aQ(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g) {
					let v = t9(),
						y = aa(v, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m);
					if (y !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, v, e, y, v[11], g, !1),
							ngDevMode && lD(n.data, o, e, nf() - 7, t, r, i, l, u, c, h, m);
					}
					return aQ;
				}
				function aK(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v, y) {
					let b = t9(),
						_ = au(b, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v);
					if (_ !== sZ) {
						let n = ne(),
							o = nO();
						ld(n, o, b, e, _, b[11], y, !1),
							ngDevMode &&
								lD(n.data, o, e, nf() - 8, t, r, i, l, u, c, h, m, v);
					}
					return aK;
				}
				function aJ(e, t, n) {
					let r = t9(),
						o = at(r, t);
					if (o !== sZ) {
						let i = ne(),
							s = nO();
						if ((ld(i, s, r, e, o, r[11], n, !1), ngDevMode)) {
							let n = [t[0]];
							for (let e = 2; e < t.length; e += 2) n.push(t[e]);
							lD(i.data, s, e, nf() - n.length + 1, ...n);
						}
					}
					return aJ;
				}
				function aX(e, t) {
					return (
						ngDevMode && L(e, 0, 32767),
						ngDevMode && L(t, 0, 32767),
						(e << 17) | (t << 2)
					);
				}
				function a0(e) {
					return ngDevMode && N(e, "expected number"), (e >> 17) & 32767;
				}
				function a1(e) {
					return ngDevMode && N(e, "expected number"), 2 | e;
				}
				function a5(e) {
					return ngDevMode && N(e, "expected number"), (131068 & e) >> 2;
				}
				function a2(e, t) {
					return (
						ngDevMode && N(e, "expected number"),
						ngDevMode && L(t, 0, 32767),
						(-131069 & e) | (t << 2)
					);
				}
				function a3(e) {
					return ngDevMode && N(e, "expected number"), 1 | e;
				}
				function a4(e, t, n, r, o) {
					let i = e[n + 1],
						s = null === t,
						l = r ? a0(i) : a5(i),
						a = !1;
					for (; 0 !== l && (!1 === a || s); ) {
						ngDevMode && K(e, l);
						let n = e[l],
							o = e[l + 1];
						(function (e, t) {
							if (
								(ngDevMode &&
									B(
										Array.isArray(t),
										!0,
										"Expected that 'tStylingKey' has been unwrapped",
									),
								null === e || null == t || (Array.isArray(e) ? e[1] : e) === t)
							)
								return !0;
							if (Array.isArray(e) && "string" == typeof t)
								return rI(e, t, 1) >= 0;
							return !1;
						})(n, t) && ((a = !0), (e[l + 1] = r ? a3(o) : a1(o))),
							(l = r ? a0(o) : a5(o));
					}
					a && (e[n + 1] = r ? a1(i) : a3(i));
				}
				let a6 = { textEnd: 0, key: 0, keyEnd: 0, value: 0, valueEnd: 0 };
				function a8(e) {
					return e.substring(a6.key, a6.keyEnd);
				}
				function a7(e, t) {
					let n = a6.textEnd;
					return n === t
						? -1
						: ((t = a6.keyEnd =
								(function (e, t, n) {
									for (; t < n && e.charCodeAt(t) > 32; ) t++;
									return t;
								})(e, (a6.key = t), n)),
						  ut(e, t, n));
				}
				function a9(e, t) {
					let n = a6.textEnd,
						r = (a6.key = ut(e, t, n));
					return n === r
						? -1
						: ((r = a6.keyEnd =
								(function (e, t, n) {
									let r;
									for (
										;
										t < n &&
										(45 === (r = e.charCodeAt(t)) ||
											95 === r ||
											((-33 & r) >= 65 && (-33 & r) <= 90) ||
											(r >= 48 && r <= 57));
									)
										t++;
									return t;
								})(e, r, n)),
						  (r = un(e, r, n, 58)),
						  (r = a6.value = ut(e, r, n)),
						  (r = a6.valueEnd =
								(function (e, t, n) {
									let r = -1,
										o = -1,
										i = -1,
										s = t,
										l = s;
									for (; s < n; ) {
										let a = e.charCodeAt(s++);
										if (59 === a) break;
										34 === a || 39 === a
											? (l = s = ur(e, a, s, n))
											: t === s - 4 &&
											  85 === i &&
											  82 === o &&
											  76 === r &&
											  40 === a
											? (l = s = ur(e, 41, s, n))
											: a > 32 && (l = s);
										(i = o), (o = r), (r = -33 & a);
									}
									return l;
								})(e, r, n)),
						  un(e, r, n, 59));
				}
				function ue(e) {
					(a6.key = 0),
						(a6.keyEnd = 0),
						(a6.value = 0),
						(a6.valueEnd = 0),
						(a6.textEnd = e.length);
				}
				function ut(e, t, n) {
					for (; t < n && 32 >= e.charCodeAt(t); ) t++;
					return t;
				}
				function un(e, t, n, r) {
					return (
						(t = ut(e, t, n)) < n &&
							(ngDevMode &&
								e.charCodeAt(t) !== r &&
								uo(e, String.fromCharCode(r), t),
							t++),
						t
					);
				}
				function ur(e, t, n, r) {
					let o = -1,
						i = n;
					for (; i < r; ) {
						let n = e.charCodeAt(i++);
						if (n == t && 92 !== o) return i;
						o = 92 == n && 92 === o ? 0 : n;
					}
					throw ngDevMode ? uo(e, String.fromCharCode(t), r) : Error();
				}
				function uo(e, t, n) {
					throw (
						(ngDevMode && V("string" == typeof e, !0, "String expected here"),
						Y(
							`Malformed style at location ${n} in string '` +
								e.substring(0, n) +
								"[>>" +
								e.substring(n, n + 1) +
								"<<]" +
								e.slice(n + 1) +
								`'. Expecting '${t}'.`,
						))
					);
				}
				function ui(e, t, n) {
					return uc(e, t, n, !1), ui;
				}
				function us(e, t) {
					return uc(e, t, null, !0), us;
				}
				function ul(e) {
					uf(uv, ua, e, !1);
				}
				function ua(e, t) {
					var n;
					for (
						let r = (ue((n = t)), a9(n, ut(n, 0, a6.textEnd)));
						r >= 0;
						r = a9(t, r)
					)
						uv(e, a8(t), t.substring(a6.value, a6.valueEnd));
				}
				function uu(e) {
					uf(uy, ud, e, !0);
				}
				function ud(e, t) {
					var n;
					for (
						let r = (ue((n = t)), a7(n, ut(n, 0, a6.textEnd)));
						r >= 0;
						r = a7(t, r)
					)
						rE(e, a8(t), !0);
				}
				function uc(e, t, n, r) {
					let o = t9(),
						i = ne(),
						s = nm(2);
					if ((i.firstUpdatePass && up(i, e, s, r), t !== sZ && l6(o, s, t))) {
						let l = i.data[nS()];
						ub(
							i,
							l,
							o,
							o[11],
							e,
							(o[s + 1] = (function (e, t) {
								return (
									null == e ||
										"" === e ||
										("string" == typeof t
											? (e += t)
											: "object" == typeof e && (e = j(ir(e)))),
									e
								);
							})(t, n)),
							r,
							s,
						);
					}
				}
				function uf(e, t, n, r) {
					let o = ne(),
						i = nm(2);
					o.firstUpdatePass && up(o, null, i, r);
					let s = t9();
					if (n !== sZ && l6(s, i, n)) {
						let l = o.data[nS()];
						if (ux(l, r) && !uh(o, i)) {
							if (ngDevMode) {
								let e = o.data[i];
								V(
									Array.isArray(e) ? e[1] : e,
									!1,
									"Styling linked list shadow input should be marked as 'false'",
								);
							}
							let e = r ? l.classesWithoutHost : l.stylesWithoutHost;
							ngDevMode &&
								!1 === r &&
								null !== e &&
								V(
									e.endsWith(";"),
									!0,
									"Expecting static portion to end with ';'",
								),
								null !== e && (n = x(e, n || "")),
								aD(o, l, s, n, r);
						} else
							(function (e, t, n, r, o, i, s, l) {
								o === sZ && (o = eA);
								let a = 0,
									u = 0,
									d = 0 < o.length ? o[0] : null,
									c = 0 < i.length ? i[0] : null;
								for (; null !== d || null !== c; ) {
									let f;
									ngDevMode && z(a, 999, "Are we stuck in infinite loop?"),
										ngDevMode && z(u, 999, "Are we stuck in infinite loop?");
									let h = a < o.length ? o[a + 1] : void 0,
										p = u < i.length ? i[u + 1] : void 0,
										m = null;
									d === c
										? ((a += 2), (u += 2), h !== p && ((m = c), (f = p)))
										: null === c || (null !== d && d < c)
										? ((a += 2), (m = d))
										: (ngDevMode && Z(c, "Expecting to have a valid key"),
										  (u += 2),
										  (m = c),
										  (f = p)),
										null !== m && ub(e, t, n, r, m, f, s, l),
										(d = a < o.length ? o[a] : null),
										(c = u < i.length ? i[u] : null);
								}
							})(
								o,
								l,
								s,
								s[11],
								s[i + 1],
								(s[i + 1] = (function (e, t, n) {
									if (null == n || "" === n) return eA;
									let r = [],
										o = ir(n);
									if (Array.isArray(o))
										for (let t = 0; t < o.length; t++) e(r, o[t], !0);
									else if ("object" == typeof o)
										for (let t in o) o.hasOwnProperty(t) && e(r, t, o[t]);
									else
										"string" == typeof o
											? t(r, o)
											: ngDevMode &&
											  Y("Unsupported styling type " + typeof o + ": " + o);
									return r;
								})(e, t, n)),
								r,
								i,
							);
					}
				}
				function uh(e, t) {
					return t >= e.expandoStartIndex;
				}
				function up(e, t, n, r) {
					ngDevMode && t_(e);
					let o = e.data;
					if (null === o[n + 1]) {
						let i = o[nS()];
						ngDevMode && Z(i, "TNode expected");
						let s = uh(e, n);
						ux(i, r) && null === t && !s && (t = !1),
							(t = (function (e, t, n, r) {
								let o = ny(e),
									i = r ? t.residualClasses : t.residualStyles;
								if (null === o) {
									let o = (r ? t.classBindings : t.styleBindings) === 0;
									o &&
										((n = ug((n = um(null, e, t, n, r)), t.attrs, r)),
										(i = null));
								} else {
									let s = t.directiveStylingLast,
										l = -1 === s || e[s] !== o;
									if (l) {
										if (((n = um(o, e, t, n, r)), null === i)) {
											let n = (function (e, t, n) {
												let r = n ? t.classBindings : t.styleBindings;
												if (0 !== a5(r)) return e[a0(r)];
											})(e, t, r);
											void 0 !== n &&
												Array.isArray(n) &&
												((n = ug((n = um(null, e, t, n[1], r)), t.attrs, r)),
												(function (e, t, n, r) {
													let o = n ? t.classBindings : t.styleBindings;
													ngDevMode &&
														B(
															a5(o),
															0,
															"Expecting to have at least one template styling binding.",
														),
														(e[a0(o)] = r);
												})(e, t, r, n));
										} else
											i = (function (e, t, n) {
												let r;
												let o = t.directiveEnd;
												ngDevMode &&
													B(
														t.directiveStylingLast,
														-1,
														"By the time this function gets called at least one hostBindings-node styling instruction must have executed.",
													);
												for (let i = 1 + t.directiveStylingLast; i < o; i++) {
													let t = e[i].hostAttrs;
													r = ug(r, t, n);
												}
												return ug(r, t.attrs, n);
											})(e, t, r);
									}
								}
								return (
									void 0 !== i &&
										(r ? (t.residualClasses = i) : (t.residualStyles = i)),
									n
								);
							})(o, i, t, r)),
							!(function (e, t, n, r, o, i) {
								let s;
								ngDevMode && t_(ne());
								let l = i ? t.classBindings : t.styleBindings,
									a = a0(l),
									u = a5(l);
								e[r] = n;
								let d = !1;
								if (Array.isArray(n)) {
									if (null === (s = n[1]) || rI(n, s, 1) > 0) d = !0;
								} else s = n;
								if (o) {
									let t = 0 !== u;
									if (t) {
										var c, f;
										let t = a0(e[a + 1]);
										(e[r + 1] = aX(t, a)),
											0 !== t && (e[t + 1] = a2(e[t + 1], r)),
											(e[a + 1] =
												((c = e[a + 1]),
												(f = r),
												ngDevMode && N(c, "expected number"),
												ngDevMode && L(f, 0, 32767),
												(131071 & c) | (f << 17)));
									} else
										(e[r + 1] = aX(a, 0)),
											0 !== a && (e[a + 1] = a2(e[a + 1], r)),
											(a = r);
								} else
									(e[r + 1] = aX(u, 0)),
										ngDevMode &&
											V(
												0 !== a && 0 === u,
												!1,
												"Adding template bindings after hostBindings is not allowed.",
											),
										0 === a ? (a = r) : (e[u + 1] = a2(e[u + 1], r)),
										(u = r);
								d && (e[r + 1] = a1(e[r + 1])),
									a4(e, s, r, !0, i),
									a4(e, s, r, !1, i),
									(function (e, t, n, r, o) {
										let i = o ? e.residualClasses : e.residualStyles;
										if (null != i && "string" == typeof t && rI(i, t, 1) >= 0)
											n[r + 1] = a3(n[r + 1]);
									})(t, s, e, r, i),
									(l = aX(a, u)),
									i ? (t.classBindings = l) : (t.styleBindings = l);
							})(o, i, t, n, s, r);
					}
				}
				function um(e, t, n, r, o) {
					let i = null,
						s = n.directiveEnd,
						l = n.directiveStylingLast;
					for (
						-1 === l ? (l = n.directiveStart) : l++;
						l < s &&
						((i = t[l]),
						ngDevMode && Z(i, "expected to be defined"),
						(r = ug(r, i.hostAttrs, o)),
						i !== e);
					) {
						l++;
					}
					return null !== e && (n.directiveStylingLast = l), r;
				}
				function ug(e, t, n) {
					let r = n ? 1 : 2,
						o = -1;
					if (null !== t)
						for (let i = 0; i < t.length; i++) {
							let s = t[i];
							"number" == typeof s
								? (o = s)
								: o === r &&
								  (!Array.isArray(e) && (e = void 0 === e ? [] : ["", e]),
								  rE(e, s, !!n || t[++i]));
						}
					return void 0 === e ? null : e;
				}
				function uv(e, t, n) {
					rE(e, t, ir(n));
				}
				function uy(e, t, n) {
					let r = String(t);
					"" !== r && !r.includes(" ") && rE(e, r, n);
				}
				function ub(e, t, n, r, o, i, s, l) {
					var a, u;
					if (!(3 & t.type)) return;
					let d = e.data,
						c = d[l + 1];
					let f = ((a = c), ngDevMode && N(a, "expected number"), (1 & a) == 1)
						? u_(d, t, n, o, a5(c), s)
						: void 0;
					if (!uj(f)) {
						if (!uj(i)) {
							if (((u = c), ngDevMode && N(u, "expected number"), (2 & u) == 2))
								i = u_(d, null, n, o, l, s);
						}
						let e = tq(nS(), n);
						!(function (e, t, n, r, o) {
							if (t)
								o
									? (ngDevMode && ngDevMode.rendererAddClass++,
									  e.addClass(n, r))
									: (ngDevMode && ngDevMode.rendererRemoveClass++,
									  e.removeClass(n, r));
							else {
								let t = -1 === r.indexOf("-") ? void 0 : r9.DashCase;
								if (null == o)
									ngDevMode && ngDevMode.rendererRemoveStyle++,
										e.removeStyle(n, r, t);
								else {
									let i = "string" == typeof o && o.endsWith("!important");
									i && ((o = o.slice(0, -10)), (t |= r9.Important)),
										ngDevMode && ngDevMode.rendererSetStyle++,
										e.setStyle(n, r, o, t);
								}
							}
						})(r, s, e, o, i);
					}
				}
				function u_(e, t, n, r, o, i) {
					let s,
						l = null === t;
					for (; o > 0; ) {
						let t = e[o],
							i = Array.isArray(t),
							a = i ? t[1] : t,
							u = null === a,
							d = n[o + 1];
						d === sZ && (d = u ? eA : void 0);
						let c = u ? rO(d, r) : a === r ? d : void 0;
						if ((i && !uj(c) && (c = rO(t, r)), uj(c) && ((s = c), l)))
							return s;
						let f = e[o + 1];
						o = l ? a0(f) : a5(f);
					}
					if (null !== t) {
						let e = i ? t.residualClasses : t.residualStyles;
						null != e && (s = rO(e, r));
					}
					return s;
				}
				function uj(e) {
					return void 0 !== e;
				}
				function ux(e, t) {
					return (e.flags & (t ? 8 : 16)) != 0;
				}
				function uD(e, t = "") {
					let n = t9(),
						r = ne(),
						o = e + 25;
					ngDevMode &&
						V(
							nf(),
							r.bindingStartIndex,
							"text nodes should be created before any bindings",
						),
						ngDevMode && K(n, o);
					let i = r.firstCreatePass ? s7(r, o, 1, t, null) : r.data[o],
						s = uw(r, n, i, t, e);
					(n[o] = s), nT && oU(r, n, s, i), ns(i, !1);
				}
				let uw = (e, t, n, r, o) => ((nT = !0), ow(t[11], r));
				function uM(e) {
					return uC("", e, ""), uM;
				}
				function uC(e, t, n) {
					let r = t9(),
						o = an(r, e, t, n);
					return o !== sZ && lO(r, nS(), o), uC;
				}
				function uS(e, t, n, r, o) {
					let i = t9(),
						s = ar(i, e, t, n, r, o);
					return s !== sZ && lO(i, nS(), s), uS;
				}
				function uE(e, t, n, r, o, i, s) {
					let l = t9(),
						a = ao(l, e, t, n, r, o, i, s);
					return a !== sZ && lO(l, nS(), a), uE;
				}
				function uO(e, t, n, r, o, i, s, l, a) {
					let u = t9(),
						d = ai(u, e, t, n, r, o, i, s, l, a);
					return d !== sZ && lO(u, nS(), d), uO;
				}
				function uA(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t9(),
						f = as(c, e, t, n, r, o, i, s, l, a, u, d);
					return f !== sZ && lO(c, nS(), f), uA;
				}
				function uI(e, t, n, r, o, i, s, l, a, u, d, c, f) {
					let h = t9(),
						p = al(h, e, t, n, r, o, i, s, l, a, u, d, c, f);
					return p !== sZ && lO(h, nS(), p), uI;
				}
				function uP(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p) {
					let m = t9(),
						g = aa(m, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p);
					return g !== sZ && lO(m, nS(), g), uP;
				}
				function uT(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g) {
					let v = t9(),
						y = au(v, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g);
					return y !== sZ && lO(v, nS(), y), uT;
				}
				function uk(e) {
					let t = t9(),
						n = at(t, e);
					return n !== sZ && lO(t, nS(), n), uk;
				}
				function uF(e, t, n) {
					let r = t9(),
						o = an(r, e, t, n);
					uf(rE, ud, o, !0);
				}
				function uR(e, t, n, r, o) {
					let i = t9(),
						s = ar(i, e, t, n, r, o);
					uf(rE, ud, s, !0);
				}
				function uN(e, t, n, r, o, i, s) {
					let l = t9(),
						a = ao(l, e, t, n, r, o, i, s);
					uf(rE, ud, a, !0);
				}
				function uL(e, t, n, r, o, i, s, l, a) {
					let u = t9(),
						d = ai(u, e, t, n, r, o, i, s, l, a);
					uf(rE, ud, d, !0);
				}
				function u$(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t9(),
						f = as(c, e, t, n, r, o, i, s, l, a, u, d);
					uf(rE, ud, f, !0);
				}
				function uV(e, t, n, r, o, i, s, l, a, u, d, c, f) {
					let h = t9(),
						p = al(h, e, t, n, r, o, i, s, l, a, u, d, c, f);
					uf(rE, ud, p, !0);
				}
				function uB(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p) {
					let m = t9(),
						g = aa(m, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p);
					uf(rE, ud, g, !0);
				}
				function uU(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g) {
					let v = t9(),
						y = au(v, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g);
					uf(rE, ud, y, !0);
				}
				function uH(e) {
					let t = t9(),
						n = at(t, e);
					uf(rE, ud, n, !0);
				}
				function uz(e, t, n) {
					let r = t9(),
						o = an(r, e, t, n);
					ul(o);
				}
				function uW(e, t, n, r, o) {
					let i = t9(),
						s = ar(i, e, t, n, r, o);
					ul(s);
				}
				function uq(e, t, n, r, o, i, s) {
					let l = t9(),
						a = ao(l, e, t, n, r, o, i, s);
					ul(a);
				}
				function uG(e, t, n, r, o, i, s, l, a) {
					let u = t9(),
						d = ai(u, e, t, n, r, o, i, s, l, a);
					ul(d);
				}
				function uZ(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t9(),
						f = as(c, e, t, n, r, o, i, s, l, a, u, d);
					ul(f);
				}
				function uY(e, t, n, r, o, i, s, l, a, u, d, c, f) {
					let h = t9(),
						p = al(h, e, t, n, r, o, i, s, l, a, u, d, c, f);
					ul(p);
				}
				function uQ(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p) {
					let m = t9(),
						g = aa(m, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p);
					ul(g);
				}
				function uK(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g) {
					let v = t9(),
						y = au(v, e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g);
					ul(y);
				}
				function uJ(e) {
					let t = t9(),
						n = at(t, e);
					ul(n);
				}
				function uX(e, t, n, r, o) {
					let i = t9(),
						s = an(i, t, n, r);
					return uc(e, s, o, !1), uX;
				}
				function u0(e, t, n, r, o, i, s) {
					let l = t9(),
						a = ar(l, t, n, r, o, i);
					return uc(e, a, s, !1), u0;
				}
				function u1(e, t, n, r, o, i, s, l, a) {
					let u = t9(),
						d = ao(u, t, n, r, o, i, s, l);
					return uc(e, d, a, !1), u1;
				}
				function u5(e, t, n, r, o, i, s, l, a, u, d) {
					let c = t9(),
						f = ai(c, t, n, r, o, i, s, l, a, u);
					return uc(e, f, d, !1), u5;
				}
				function u2(e, t, n, r, o, i, s, l, a, u, d, c, f) {
					let h = t9(),
						p = as(h, t, n, r, o, i, s, l, a, u, d, c);
					return uc(e, p, f, !1), u2;
				}
				function u3(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p) {
					let m = t9(),
						g = al(m, t, n, r, o, i, s, l, a, u, d, c, f, h);
					return uc(e, g, p, !1), u3;
				}
				function u4(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g) {
					let v = t9(),
						y = aa(v, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m);
					return uc(e, y, g, !1), u4;
				}
				function u6(e, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v, y) {
					let b = t9(),
						_ = au(b, t, n, r, o, i, s, l, a, u, d, c, f, h, p, m, g, v);
					return uc(e, _, y, !1), u6;
				}
				function u8(e, t, n) {
					let r = t9(),
						o = at(r, t);
					return uc(e, o, n, !1), u8;
				}
				function u7(e, t, n) {
					let r = t9(),
						o = np();
					if (l6(r, o, t)) {
						let i = ne(),
							s = nO();
						ld(i, s, r, e, t, r[11], n, !0), ngDevMode && lD(i.data, s, e, o);
					}
					return u7;
				}
				function u9(e, t, n) {
					let r = t9(),
						o = np();
					if (l6(r, o, t)) {
						let i = ne(),
							s = nO(),
							l = ny(i.data),
							a = lC(l, s, r);
						ld(i, s, r, e, t, a, n, !0), ngDevMode && lD(i.data, s, e, o);
					}
					return u9;
				}
				"undefined" == typeof ngI18nClosureMode &&
					(ec.ngI18nClosureMode =
						"undefined" != typeof goog && "function" == typeof goog.getMsg);
				let de = void 0;
				var dt = [
					"en",
					[["a", "p"], ["AM", "PM"], de],
					[["AM", "PM"], de, de],
					[
						["S", "M", "T", "W", "T", "F", "S"],
						["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
						[
							"Sunday",
							"Monday",
							"Tuesday",
							"Wednesday",
							"Thursday",
							"Friday",
							"Saturday",
						],
						["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
					],
					de,
					[
						["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
						[
							"Jan",
							"Feb",
							"Mar",
							"Apr",
							"May",
							"Jun",
							"Jul",
							"Aug",
							"Sep",
							"Oct",
							"Nov",
							"Dec",
						],
						[
							"January",
							"February",
							"March",
							"April",
							"May",
							"June",
							"July",
							"August",
							"September",
							"October",
							"November",
							"December",
						],
					],
					de,
					[
						["B", "A"],
						["BC", "AD"],
						["Before Christ", "Anno Domini"],
					],
					0,
					[6, 0],
					["M/d/yy", "MMM d, y", "MMMM d, y", "EEEE, MMMM d, y"],
					["h:mm a", "h:mm:ss a", "h:mm:ss a z", "h:mm:ss a zzzz"],
					["{1}, {0}", de, "{1} 'at' {0}", de],
					[".", ",", ";", "%", "+", "-", "E", "\xd7", "‰", "∞", "NaN", ":"],
					["#,##0.###", "#,##0%", "\xa4#,##0.00", "#E0"],
					"USD",
					"$",
					"US Dollar",
					{},
					"ltr",
					function (e) {
						let t = e.toString().replace(/^[^.]*\.?/, "").length;
						return 1 === Math.floor(Math.abs(e)) && 0 === t ? 1 : 5;
					},
				];
				let dn = {};
				function dr(e) {
					let t = (function (e) {
							return e.toLowerCase().replace(/_/g, "-");
						})(e),
						n = ds(t);
					if (n) return n;
					let r = t.split("-")[0];
					if ((n = ds(r))) return n;
					if ("en" === r) return dt;
					throw new O(
						701,
						ngDevMode && `Missing locale data for the locale "${e}".`,
					);
				}
				function di(e) {
					let t = dr(e);
					return t[dl.PluralCase];
				}
				function ds(e) {
					return (
						!(e in dn) &&
							(dn[e] =
								ec.ng &&
								ec.ng.common &&
								ec.ng.common.locales &&
								ec.ng.common.locales[e]),
						dn[e]
					);
				}
				var dl =
					(((dl = dl || {})[(dl.LocaleId = 0)] = "LocaleId"),
					(dl[(dl.DayPeriodsFormat = 1)] = "DayPeriodsFormat"),
					(dl[(dl.DayPeriodsStandalone = 2)] = "DayPeriodsStandalone"),
					(dl[(dl.DaysFormat = 3)] = "DaysFormat"),
					(dl[(dl.DaysStandalone = 4)] = "DaysStandalone"),
					(dl[(dl.MonthsFormat = 5)] = "MonthsFormat"),
					(dl[(dl.MonthsStandalone = 6)] = "MonthsStandalone"),
					(dl[(dl.Eras = 7)] = "Eras"),
					(dl[(dl.FirstDayOfWeek = 8)] = "FirstDayOfWeek"),
					(dl[(dl.WeekendRange = 9)] = "WeekendRange"),
					(dl[(dl.DateFormat = 10)] = "DateFormat"),
					(dl[(dl.TimeFormat = 11)] = "TimeFormat"),
					(dl[(dl.DateTimeFormat = 12)] = "DateTimeFormat"),
					(dl[(dl.NumberSymbols = 13)] = "NumberSymbols"),
					(dl[(dl.NumberFormats = 14)] = "NumberFormats"),
					(dl[(dl.CurrencyCode = 15)] = "CurrencyCode"),
					(dl[(dl.CurrencySymbol = 16)] = "CurrencySymbol"),
					(dl[(dl.CurrencyName = 17)] = "CurrencyName"),
					(dl[(dl.Currencies = 18)] = "Currencies"),
					(dl[(dl.Directionality = 19)] = "Directionality"),
					(dl[(dl.PluralCase = 20)] = "PluralCase"),
					(dl[(dl.ExtraData = 21)] = "ExtraData"),
					dl);
				let da = ["zero", "one", "two", "few", "many"],
					du = "en-US",
					dd = { marker: "element" },
					dc = { marker: "ICU" };
				var df =
					(((df = df || {})[(df.SHIFT = 2)] = "SHIFT"),
					(df[(df.APPEND_EAGERLY = 1)] = "APPEND_EAGERLY"),
					(df[(df.COMMENT = 2)] = "COMMENT"),
					df);
				let dh = du;
				function dp(e, t, n) {
					let r = t.insertBeforeIndex,
						o = Array.isArray(r) ? r[0] : r;
					return null === o ? o$(e, t, n) : (ngDevMode && K(n, o), tW(n[o]));
				}
				function dm(e, t, n, r, o) {
					let i = t.insertBeforeIndex;
					if (Array.isArray(i)) {
						ngDevMode && Q(r);
						let s = r,
							l = null;
						if (
							(!(3 & t.type) && ((l = s), (s = o)),
							null !== s && -1 === t.componentOffset)
						)
							for (let t = 1; t < i.length; t++) {
								let r = n[i[t]];
								ok(e, s, r, l, !1);
							}
					}
				}
				function dg(e, t) {
					if (
						(ngDevMode &&
							V(
								t.insertBeforeIndex,
								null,
								"We expect that insertBeforeIndex is not set",
							),
						e.push(t),
						e.length > 1)
					)
						for (let n = e.length - 2; n >= 0; n--) {
							let r = e[n];
							!(function (e) {
								return !(64 & e.type);
							})(r) &&
								(function (e, t) {
									return !(64 & t.type) || e.index > t.index;
								})(r, t) &&
								null ===
									(function (e) {
										let t = e.insertBeforeIndex;
										return Array.isArray(t) ? t[0] : t;
									})(r) &&
								(function (e, t) {
									let n = e.insertBeforeIndex;
									if (Array.isArray(n)) n[0] = t;
									else {
										var r, o;
										(r = dp),
											(o = dm),
											(oV = r),
											(i = o),
											(e.insertBeforeIndex = t);
									}
								})(r, t.index);
						}
				}
				function dv(e) {
					return !(64 & e.type);
				}
				function dy(e, t) {
					let n = e.data[t];
					if (null === n || "string" == typeof n) return null;
					ngDevMode &&
						!(
							n.hasOwnProperty("tView") ||
							n.hasOwnProperty("currentCaseLViewIndex")
						) &&
						Y("We expect to get 'null'|'TIcu'|'TIcuContainer', but got: " + n);
					let r = n.hasOwnProperty("currentCaseLViewIndex") ? n : n.value;
					return ngDevMode && tp(r), r;
				}
				function db(e, t) {
					let n = t[e.currentCaseLViewIndex];
					return null === n ? n : n < 0 ? ~n : n;
				}
				function d_(e) {
					return e >>> 17;
				}
				function dj(e) {
					return (131070 & e) >>> 1;
				}
				function dx(e) {
					return 1 & e;
				}
				let dD = 0,
					dw = 0;
				function dM(e, t, n, r, o) {
					for (let i = 0; i < n.length; i++) {
						let s = n[i],
							l = n[++i];
						if (s & o) {
							let o = "";
							for (let s = i + 1; s <= i + l; s++) {
								let i = n[s];
								if ("string" == typeof i) o += i;
								else if ("number" == typeof i) {
									if (i < 0) o += I(t[r - i]);
									else {
										let l = i >>> 2;
										switch (3 & i) {
											case 1:
												let a = n[++s],
													u = n[++s],
													d = e.data[l];
												ngDevMode && Z(d, "Experting TNode or string"),
													"string" == typeof d
														? lv(t[11], t[l], null, d, a, o, u)
														: ld(e, d, t, a, o, t[11], u, !1);
												break;
											case 0:
												let c = t[l];
												null !== c && oM(t[11], c, o);
												break;
											case 2:
												(function (e, t, n, r) {
													let o = (function (e, t) {
														let n = e.cases.indexOf(t);
														if (-1 === n)
															switch (e.type) {
																case 1: {
																	let r = (function (e, t) {
																		let n = di(t)(parseInt(e, 10)),
																			r = da[n];
																		return void 0 !== r ? r : "other";
																	})(t, dh);
																	-1 === (n = e.cases.indexOf(r)) &&
																		"other" !== r &&
																		(n = e.cases.indexOf("other"));
																	break;
																}
																case 0:
																	n = e.cases.indexOf("other");
															}
														return -1 === n ? null : n;
													})(t, r);
													if (
														db(t, n) !== o &&
														((function e(t, n, r) {
															let o = db(n, r);
															if (null !== o) {
																let i = n.remove[o];
																for (let n = 0; n < i.length; n++) {
																	let o = i[n];
																	if (o > 0) {
																		let e = tq(o, r);
																		null !== e && oW(r[11], e);
																	} else e(t, dy(t, ~o), r);
																}
															}
														})(e, t, n),
														(n[t.currentCaseLViewIndex] =
															null === o ? null : ~o),
														null !== o)
													) {
														let r = n[t.anchorIdx];
														r &&
															(ngDevMode && Q(r),
															!(function e(t, n, r, o) {
																let i;
																ngDevMode && Q(o);
																let s = r[11],
																	l = null;
																for (let a = 0; a < n.length; a++) {
																	let u = n[a];
																	if ("string" == typeof u) {
																		let e = n[++a];
																		null === r[e] &&
																			(ngDevMode &&
																				ngDevMode.rendererCreateTextNode++,
																			ngDevMode && K(r, e),
																			(r[e] = ow(s, u)));
																	} else if ("number" == typeof u)
																		switch (1 & u) {
																			case 0:
																				let d, c;
																				let f = u >>> 17;
																				if (
																					(null === l &&
																						((l = f), (i = oN(s, o))),
																					f === l
																						? ((d = o), (c = i))
																						: ((d = null), (c = tW(r[f]))),
																					null !== c)
																				) {
																					ngDevMode && Q(c);
																					let n = dj(u);
																					ngDevMode && q(n, 25, "Missing ref");
																					let o = r[n];
																					ngDevMode && Q(o), ok(s, c, o, d, !1);
																					let i = dy(t, n);
																					if (
																						null !== i &&
																						"object" == typeof i
																					) {
																						ngDevMode && tp(i);
																						let n = db(i, r);
																						null !== n &&
																							e(
																								t,
																								i.create[n],
																								r,
																								r[i.anchorIdx],
																							);
																					}
																				}
																				break;
																			case 1:
																				let h = u >>> 1,
																					p = n[++a],
																					m = n[++a];
																				lv(s, tq(h, r), null, null, p, m, null);
																				break;
																			default:
																				if (ngDevMode)
																					throw new O(
																						700,
																						`Unable to determine the type of mutate operation for "${u}"`,
																					);
																		}
																	else
																		switch (u) {
																			case dc:
																				let g = n[++a],
																					v = n[++a];
																				if (null === r[v]) {
																					ngDevMode &&
																						V(
																							typeof g,
																							"string",
																							`Expected "${g}" to be a comment node value`,
																						),
																						ngDevMode &&
																							ngDevMode.rendererCreateComment++,
																						ngDevMode && tj(r, v);
																					let e = (r[v] = oC(s, g));
																					oc(e, r);
																				}
																				break;
																			case dd:
																				let y = n[++a],
																					b = n[++a];
																				if (null === r[b]) {
																					ngDevMode &&
																						V(
																							typeof y,
																							"string",
																							`Expected "${y}" to be an element node tag name`,
																						),
																						ngDevMode &&
																							ngDevMode.rendererCreateElement++,
																						ngDevMode && tj(r, b);
																					let e = (r[b] = oS(s, y, null));
																					oc(e, r);
																				}
																				break;
																			default:
																				ngDevMode &&
																					Y(
																						`Unable to determine the type of mutate operation for "${u}"`,
																					);
																		}
																}
															})(e, t.create[o], n, r));
													}
												})(e, dy(e, l), t, o);
												break;
											case 3:
												dC(e, dy(e, l), r, t);
										}
									}
								}
							}
						} else {
							let o = n[i + 1];
							if (o > 0 && (3 & o) == 3) {
								let n = o >>> 2,
									i = dy(e, n),
									s = t[i.currentCaseLViewIndex];
								s < 0 && dC(e, i, r, t);
							}
						}
						i += l;
					}
				}
				function dC(e, t, n, r) {
					ngDevMode && K(r, t.currentCaseLViewIndex);
					let o = r[t.currentCaseLViewIndex];
					if (null !== o) {
						let i = dD;
						o < 0 && ((o = r[t.currentCaseLViewIndex] = ~o), (i = -1)),
							dM(e, r, t.update[o], n, i);
					}
				}
				function dS() {
					let e, t;
					let n = [],
						r = -1;
					function o(e, n) {
						r = 0;
						let o = db(e, n);
						null !== o
							? (ngDevMode && L(o, 0, e.cases.length - 1), (t = e.remove[o]))
							: (t = eA);
					}
					return function (i, s) {
						for (e = s; n.length; ) n.pop();
						return (
							ngDevMode && tc(i, s),
							o(i.value, s),
							function i() {
								if (r < t.length) {
									let s = t[r++];
									if ((ngDevMode && N(s, "Expecting OpCode number"), s > 0)) {
										let t = e[s];
										return ngDevMode && Q(t), t;
									}
									{
										n.push(r, t);
										let l = ~s,
											a = e[1].data[l];
										return ngDevMode && tp(a), o(a, e), i();
									}
								}
								return 0 === n.length
									? null
									: ((t = n.pop()), (r = n.pop()), i());
							}
						);
					};
				}
				function dE(e) {
					let t = e || (Array.isArray(this) ? this : []),
						n = [];
					for (let e = 0; e < t.length; e++) {
						let r = t[e++],
							o = t[e],
							i = (r & df.COMMENT) === df.COMMENT,
							s = (r & df.APPEND_EAGERLY) === df.APPEND_EAGERLY,
							l = r >>> df.SHIFT;
						n.push(
							`lView[${l}] = document.${
								i ? "createComment" : "createText"
							}(${JSON.stringify(o)});`,
						),
							s && n.push(`parent.appendChild(lView[${l}]);`);
					}
					return n;
				}
				function dO(e) {
					let t = new dP(e || (Array.isArray(this) ? this : [])),
						n = [];
					for (; t.hasMore(); ) {
						let e = t.consumeNumber(),
							r = t.consumeNumber(),
							o = t.i + r,
							i = [],
							s = "";
						for (; t.i < o; ) {
							let e = t.consumeNumberOrString();
							if ("string" == typeof e) s += e;
							else if (e < 0) s += "${lView[i" + e + "]}";
							else {
								let n = (function (e) {
									let n = e >>> 2;
									switch (3 & e) {
										case 0:
											return `(lView[${n}] as Text).textContent = $$$`;
										case 1:
											let r = t.consumeString(),
												o = t.consumeFunction(),
												i = o ? `(${o})($$$)` : "$$$";
											return `(lView[${n}] as Element).setAttribute('${r}', ${i})`;
										case 2:
											return `icuSwitchCase(${n}, $$$)`;
										case 3:
											return `icuUpdateCase(${n})`;
									}
									throw Error("unexpected OpCode");
								})(e);
								i.push(n.replace("$$$", "`" + s + "`") + ";"), (s = "");
							}
						}
						n.push(`if (mask & 0b${e.toString(2)}) { ${i.join(" ")} }`);
					}
					return n;
				}
				function dA(e) {
					let t = new dP(e || (Array.isArray(this) ? this : [])),
						n = [],
						r = -1;
					for (; t.hasMore(); ) {
						let e = t.consumeNumberStringOrMarker();
						if (e === dc) {
							let e = t.consumeString();
							(r = t.consumeNumber()),
								n.push(`lView[${r}] = document.createComment("${e}")`);
						} else if (e === dd) {
							let e = t.consumeString();
							(r = t.consumeNumber()),
								n.push(`lView[${r}] = document.createElement("${e}")`);
						} else if ("string" == typeof e)
							(r = t.consumeNumber()),
								n.push(`lView[${r}] = document.createTextNode("${e}")`);
						else if ("number" == typeof e) {
							let o = (function (e) {
								let n = e >>> 17,
									o = dj(e);
								switch (1 & e) {
									case 0:
										return `(lView[${n}] as Element).appendChild(lView[${r}])`;
									case 1:
										return `(lView[${o}] as Element).setAttribute("${t.consumeString()}", "${t.consumeString()}")`;
								}
								throw Error("Unexpected OpCode: " + (1 & e));
							})(e);
							o && n.push(o);
						} else throw Error("Unexpected value");
					}
					return n;
				}
				function dI(e) {
					let t = e || (Array.isArray(this) ? this : []),
						n = [];
					for (let e = 0; e < t.length; e++) {
						let r = t[e];
						r > 0
							? n.push(`remove(lView[${r}])`)
							: n.push(`removeNestedICU(${~r})`);
					}
					return n;
				}
				class dP {
					hasMore() {
						return this.i < this.codes.length;
					}
					consumeNumber() {
						let e = this.codes[this.i++];
						return N(e, "expecting number in OpCode"), e;
					}
					consumeString() {
						let e = this.codes[this.i++];
						return $(e, "expecting string in OpCode"), e;
					}
					consumeFunction() {
						let e = this.codes[this.i++];
						if (null === e || "function" == typeof e) return e;
						throw Error("expecting function in OpCode");
					}
					consumeNumberOrString() {
						let e = this.codes[this.i++];
						return "string" == typeof e
							? e
							: (N(e, "expecting number or string in OpCode"), e);
					}
					consumeNumberStringOrMarker() {
						let e = this.codes[this.i++];
						return "string" == typeof e ||
							"number" == typeof e ||
							e == dc ||
							e == dd
							? e
							: (N(
									e,
									"expecting number, string, ICU_MARKER or ELEMENT_MARKER in OpCode",
							  ),
							  e);
					}
					constructor(e) {
						(this.i = 0), (this.codes = e);
					}
				}
				let dT = /�(\d+):?\d*�/gi,
					dk = /({\s*�\d+:?\d*�\s*,\s*\S{6}\s*,[\s\S]*})/gi,
					dF = /�(\d+)�/,
					dR = /^\s*(�\d+:?\d*�)\s*,\s*(select|plural)\s*,/,
					dN = `�`,
					dL = /�\/?\*(\d+:\d+)�/gi,
					d$ = /�(\/?[#*]\d+):?\d*�/gi,
					dV = /\uE500/g;
				function dB(e, t) {
					if (ngDevMode)
						Object.defineProperty(e, "debug", { get: t, enumerable: !1 });
					else
						throw Error(
							"This method should be guarded with `ngDevMode` so that it can be tree shaken in production!",
						);
				}
				function dU(e, t, n, r, s, l, a) {
					let u = le(e, r, 1, null),
						d = u << df.SHIFT,
						c = ni();
					if (
						(t === c && (c = null), null === c && (d |= df.APPEND_EAGERLY), a)
					) {
						var f;
						(d |= df.COMMENT), (f = dS), void 0 === o && (o = f());
					}
					s.push(d, null === l ? "" : l);
					let h = s9(
						e,
						u,
						a ? 32 : 1,
						null === l ? (ngDevMode ? "{{?}}" : "") : l,
						null,
					);
					dg(n, h);
					let p = h.index;
					return (
						ns(h, !1),
						null !== c &&
							t !== c &&
							!(function (e, t) {
								ngDevMode && th(e);
								let n = e.insertBeforeIndex;
								if (null === n) {
									var r, o;
									(r = dp),
										(o = dm),
										(oV = r),
										(i = o),
										(n = e.insertBeforeIndex = [null, t]);
								} else
									V(Array.isArray(n), !0, "Expecting array here"), n.push(t);
							})(c, p),
						h
					);
				}
				function dH(e, t, n, r, o, i) {
					ngDevMode && G(n, 25, "Index must be in absolute LView offset");
					let s = e.length;
					e.push(null, null);
					ngDevMode && dB(e, dO);
					let l = t.split(dT),
						a = 0;
					for (let t = 0; t < l.length; t++) {
						let n = l[t];
						if (1 & t) {
							let t = o + parseInt(n, 10);
							e.push(-1 - t), (a |= dz(t));
						} else "" !== n && e.push(n);
					}
					return (
						e.push((n << 2) | (r ? 1 : 0)),
						r && e.push(r, i),
						(e[s] = a),
						(e[s + 1] = e.length - (s + 2)),
						a
					);
				}
				function dz(e) {
					return 1 << Math.min(e, 31);
				}
				function dW(e) {
					let t, n;
					let r = "",
						o = 0,
						i = !1;
					for (; null !== (t = dL.exec(e)); )
						i
							? t[0] === `${dN}/*${n}${dN}` && ((o = t.index), (i = !1))
							: ((r += e.substring(o, t.index + t[0].length)),
							  (n = t[1]),
							  (i = !0));
					return (
						ngDevMode &&
							V(
								i,
								!1,
								`Tag mismatch: unable to find the end of the sub-template in the translation "${e}"`,
							),
						(r += e.slice(o))
					);
				}
				function dq(e, t, n, r, o, i) {
					ngDevMode && Z(o, "ICU expression must be defined");
					let s = 0,
						l = {
							type: o.type,
							currentCaseLViewIndex: le(e, t, 1, null),
							anchorIdx: i,
							cases: [],
							create: [],
							remove: [],
							update: [],
						};
					(function (e, t, n) {
						e.push(dz(t.mainBinding), 2, -1 - t.mainBinding, (n << 2) | 2);
					})(n, o, i),
						!(function (e, t, n) {
							let r = e.data[t];
							ngDevMode &&
								V(
									null === r || r.hasOwnProperty("tView"),
									!0,
									"We expect to get 'null'|'TIcuContainer'",
								),
								null === r
									? (e.data[t] = n)
									: (ngDevMode && nz(r, 32), (r.value = n));
						})(e, i, l);
					let a = o.values;
					for (let i = 0; i < a.length; i++) {
						let u = a[i],
							d = [];
						for (let e = 0; e < u.length; e++) {
							let t = u[e];
							if ("string" != typeof t) {
								let n = d.push(t) - 1;
								u[e] = `<!--�${n}�-->`;
							}
						}
						s =
							(function (e, t, n, r, o, i, s, l) {
								let a = [],
									u = [],
									d = [];
								ngDevMode && (dB(a, dA), dB(u, dI), dB(d, dO)),
									t.cases.push(i),
									t.create.push(a),
									t.remove.push(u),
									t.update.push(d);
								let c = id(o1()),
									f = c.getInertBodyElement(s);
								ngDevMode && Z(f, "Unable to generate inert body element");
								let h = iF(f) || f;
								return h
									? (function e(t, n, r, o, i, s, l, a, u, d, c) {
											let f = 0,
												h = a.firstChild;
											for (; h; ) {
												let a = le(t, r, 1, null);
												switch (h.nodeType) {
													case Node.ELEMENT_NODE:
														let p = h,
															m = p.tagName.toLowerCase();
														if (iw.hasOwnProperty(m)) {
															dY(i, dd, m, u, a), (t.data[a] = m);
															let g = p.attributes;
															for (let e = 0; e < g.length; e++) {
																let t = g.item(e),
																	n = t.name.toLowerCase(),
																	r = !!t.value.match(dT);
																r
																	? iE.hasOwnProperty(n)
																		? iM[n]
																			? dH(l, t.value, a, t.name, 0, im)
																			: dH(l, t.value, a, t.name, 0, null)
																		: ngDevMode &&
																		  console.warn(
																				`WARNING: ignoring unsafe attribute value ${n} on element ${m} (see ${E})`,
																		  )
																	: (function (e, t, n) {
																			e.push((t << 1) | 1, n.name, n.value);
																	  })(i, a, t);
															}
															(f = e(t, n, r, o, i, s, l, h, a, d, c + 1) | f),
																dZ(s, a, c);
														}
														break;
													case Node.TEXT_NODE:
														let g = h.textContent || "",
															v = g.match(dT);
														dY(i, null, v ? "" : g, u, a),
															dZ(s, a, c),
															v && (f = dH(l, g, a, null, 0, null) | f);
														break;
													case Node.COMMENT_NODE:
														let y = dF.exec(h.textContent || "");
														if (y) {
															let e = parseInt(y[1], 10),
																n = d[e];
															dY(
																i,
																dc,
																ngDevMode ? `nested ICU ${e}` : "",
																u,
																a,
															),
																dq(t, r, o, u, n, a),
																(function (e, t, n) {
																	0 === n && (e.push(~t), e.push(t));
																})(s, a, c);
														}
												}
												h = h.nextSibling;
											}
											return f;
									  })(e, t, n, r, a, u, d, h, o, l, 0)
									: 0;
							})(e, l, t, n, r, o.cases[i], u.join(""), d) | s;
					}
					s &&
						(function (e, t, n) {
							e.push(t, 1, (n << 2) | 3);
						})(n, s, i);
				}
				function dG(e) {
					let t;
					if (!e) return [];
					let n = 0,
						r = [],
						o = [],
						i = /[{}]/g;
					for (i.lastIndex = 0; (t = i.exec(e)); ) {
						let i = t.index;
						if ("}" == t[0]) {
							if ((r.pop(), 0 == r.length)) {
								let t = e.substring(n, i);
								dR.test(t)
									? o.push(
											(function (e) {
												let t = [],
													n = [],
													r = 1,
													o = 0;
												e = e.replace(dR, function (e, t, n) {
													return (
														(r = "select" === n ? 0 : 1),
														(o = parseInt(t.slice(1), 10)),
														""
													);
												});
												let i = dG(e);
												for (let e = 0; e < i.length; ) {
													let o = i[e++].trim();
													1 === r && (o = o.replace(/\s*(?:=)?(\w+)\s*/, "$1")),
														o.length && t.push(o);
													let s = dG(i[e++]);
													t.length > n.length && n.push(s);
												}
												return { type: r, mainBinding: o, cases: t, values: n };
											})(t),
									  )
									: o.push(t),
									(n = i + 1);
							}
						} else {
							if (0 == r.length) {
								let t = e.substring(n, i);
								o.push(t), (n = i + 1);
							}
							r.push("{");
						}
					}
					let s = e.substring(n);
					return o.push(s), o;
				}
				function dZ(e, t, n) {
					0 === n && e.push(t);
				}
				function dY(e, t, n, r, o) {
					var i, s;
					null !== t && e.push(t),
						e.push(
							n,
							o,
							((i = r),
							(s = o),
							ngDevMode && G(i, 0, "Missing parent index"),
							ngDevMode && q(s, 0, "Missing ref index"),
							0 | (i << 17) | (s << 1)),
						);
				}
				let dQ = /\[(�.+?�?)\]/,
					dK = /\[(�.+?�?)\]|(�\/?\*\d+:\d+�)/g,
					dJ = /({\s*)(VAR_(PLURAL|SELECT)(_\d+)?)(\s*,)/g,
					dX = /{([A-Z0-9_]+)}/g,
					d0 = /�I18N_EXP_(ICU(_\d+)?)�/g,
					d1 = /\/\*/,
					d5 = /\d+\:(\d+)/;
				function d2(e, t, n = -1) {
					let r = ne(),
						o = t9(),
						i = 25 + e;
					ngDevMode && Z(r, "tView should be defined");
					let s = tX(r.consts, t),
						l = ni();
					if (
						(r.firstCreatePass &&
							!(function (e, t, n, r, o, i) {
								let s = ni(),
									l = [],
									a = [],
									u = [[]];
								ngDevMode && (dB(l, dE), dB(a, dO)),
									(o = (function (e, t) {
										if (-1 === t) return dW(e);
										{
											let n = e.indexOf(`:${t}${dN}`) + 2 + t.toString().length,
												r = e.search(RegExp(`${dN}\\/\\*\\d+:${t}${dN}`));
											return dW(e.substring(n, r));
										}
									})(o, i));
								let d = o.replace(dV, " ").split(d$);
								for (let i = 0; i < d.length; i++) {
									let c = d[i];
									if ((1 & i) == 0) {
										let i = dG(c);
										for (let d = 0; d < i.length; d++) {
											let c = i[d];
											if ((1 & d) == 0)
												ngDevMode && $(c, "Parsed ICU part should be string"),
													"" !== c &&
														(function (e, t, n, r, o, i, s) {
															let l = s.match(dT),
																a = dU(e, t, n, i, r, l ? null : s, !1);
															l && dH(o, s, a.index, null, 0, null);
														})(e, s, u[0], l, a, n, c);
											else {
												if ("object" != typeof c)
													throw Error(
														`Unable to parse ICU expression in "${o}" message.`,
													);
												let i = dU(
														e,
														s,
														u[0],
														n,
														l,
														ngDevMode ? `ICU ${r}:${c.mainBinding}` : "",
														!0,
													),
													d = i.index;
												ngDevMode &&
													G(d, 25, "Index must be in absolute LView offset"),
													dq(e, n, a, t, c, d);
											}
										}
									} else {
										let t = 47 === c.charCodeAt(0),
											n = c.charCodeAt(t ? 1 : 0);
										ngDevMode &&
											(function (e, ...t) {
												if (-1 !== t.indexOf(e)) return;
												Y(
													`Expected value to be one of ${JSON.stringify(
														t,
													)} but was ${JSON.stringify(e)}.`,
												);
											})(n, 42, 35);
										let r = 25 + Number.parseInt(c.substring(t ? 2 : 1));
										if (t) u.shift(), ns(ni(), !1);
										else {
											let t = (function (e, t, n) {
												let r = s9(e, n, 64, null, null);
												return dg(t, r), r;
											})(e, u[0], r);
											u.unshift([]), ns(t, !0);
										}
									}
								}
								e.data[r] = { create: l, update: a };
							})(r, null === l ? 0 : l.index, o, i, s, n),
						2 === r.type)
					) {
						let e = o[15];
						e[2] |= 32;
					} else o[2] |= 32;
					let a = r.data[i],
						u = l === o[6] ? null : l,
						d = oT(r, u, o),
						c = l && 8 & l.type ? o[l.index] : null;
					!(function (e, t, n, r) {
						let o = e[11];
						for (let i = 0; i < t.length; i++) {
							let s = t[i++],
								l = t[i],
								a = (s & df.COMMENT) === df.COMMENT,
								u = (s & df.APPEND_EAGERLY) === df.APPEND_EAGERLY,
								d = s >>> df.SHIFT,
								c = e[d];
							null === c && (c = e[d] = a ? o.createComment(l) : ow(o, l)),
								u && null !== n && ok(o, n, c, r, !1);
						}
					})(o, a.create, d, c),
						ng(!0);
				}
				function d3() {
					ng(!1);
				}
				function d4(e, t, n) {
					d2(e, t, n), d3();
				}
				function d6(e, t) {
					let n = ne();
					ngDevMode && Z(n, "tView should be defined");
					let r = tX(n.consts, t);
					!(function (e, t, n) {
						let r = nr(),
							o = r.index,
							i = [];
						if (
							(ngDevMode && dB(i, dO), e.firstCreatePass && null === e.data[t])
						) {
							for (let e = 0; e < n.length; e += 2) {
								let t = n[e],
									r = n[e + 1];
								if ("" !== r) {
									if (dk.test(r))
										throw Error(
											`ICU expressions are not supported in attributes. Message: "${r}".`,
										);
									dH(
										i,
										r,
										o,
										t,
										(function (e) {
											let t = 0;
											for (let n = 0; n < e.length; n++) {
												let r = e[n];
												"number" == typeof r && r < 0 && t++;
											}
											return t;
										})(i),
										null,
									);
								}
							}
							e.data[t] = i;
						}
					})(n, e + 25, r);
				}
				function d8(e) {
					let t = t9();
					return l6(t, np(), e) && (dD |= 1 << Math.min(dw, 31)), dw++, d8;
				}
				function d7(e) {
					!(function (e, t, n) {
						if (dw > 0) {
							ngDevMode && Z(e, "tView should be defined");
							let r = e.data[n],
								o = Array.isArray(r) ? r : r.update,
								i = nf() - dw - 1;
							dM(e, t, o, i, dD);
						}
						(dD = 0), (dw = 0);
					})(ne(), t9(), e + 25);
				}
				function d9(e, t = {}) {
					return (function (e, t = {}) {
						let n = e;
						if (dQ.test(e)) {
							let e = {},
								t = [0];
							n = n.replace(dK, (n, r, o) => {
								let i = r || o,
									s = e[i] || [];
								if (
									(!s.length &&
										(i.split("|").forEach((e) => {
											let t = e.match(d5),
												n = t ? parseInt(t[1], 10) : 0,
												r = d1.test(e);
											s.push([n, r, e]);
										}),
										(e[i] = s)),
									!s.length)
								)
									throw Error(`i18n postprocess: unmatched placeholder - ${i}`);
								let l = t[t.length - 1],
									a = 0;
								for (let e = 0; e < s.length; e++)
									if (s[e][0] === l) {
										a = e;
										break;
									}
								let [u, d, c] = s[a];
								return d ? t.pop() : l !== u && t.push(u), s.splice(a, 1), c;
							});
						}
						return Object.keys(t).length
							? (n = (n = (n = n.replace(dJ, (e, n, r, o, i, s) =>
									t.hasOwnProperty(r) ? `${n}${t[r]}${s}` : e,
							  )).replace(dX, (e, n) =>
									t.hasOwnProperty(n) ? t[n] : e,
							  )).replace(d0, (e, n) => {
									if (t.hasOwnProperty(n)) {
										let r = t[n];
										if (!r.length)
											throw Error(
												`i18n postprocess: unmatched ICU - ${e} with key: ${n}`,
											);
										return r.shift();
									}
									return e;
							  }))
							: n;
					})(e, t);
				}
				function ce(e, t, n, r, o) {
					if (Array.isArray((e = M(e))))
						for (let i = 0; i < e.length; i++) ce(e[i], t, n, r, o);
					else {
						let i = ne(),
							s = t9(),
							l = i6(e) ? e : M(e.provide),
							a = so(e),
							u = nr(),
							d = 1048575 & u.providerIndexes,
							c = u.directiveStart,
							f = u.providerIndexes >> 20;
						if (i6(e) || !e.multi) {
							let r = new nU(a, o, s4),
								h = cr(l, t, o ? d : d + f, c);
							-1 === h
								? (n2(nX(u, s), i, l),
								  ct(i, e, t.length),
								  t.push(l),
								  u.directiveStart++,
								  u.directiveEnd++,
								  o && (u.providerIndexes += 1048576),
								  n.push(r),
								  s.push(r))
								: ((n[h] = r), (s[h] = r));
						} else {
							let h = cr(l, t, d + f, c),
								p = cr(l, t, d, d + f),
								m = h >= 0 && n[h],
								g = p >= 0 && n[p];
							if ((!o || g) && (o || m)) {
								let t = cn(n[o ? p : h], a, !o && r);
								ct(i, e, h > -1 ? h : p, t);
							} else {
								n2(nX(u, s), i, l);
								let d = (function (e, t, n, r, o) {
									let i = new nU(e, n, s4);
									return (
										(i.multi = []),
										(i.index = t),
										(i.componentProviders = 0),
										cn(i, o, r && !n),
										i
									);
								})(o ? ci : co, n.length, o, r, a);
								!o && g && (n[p].providerFactory = d),
									ct(i, e, t.length, 0),
									t.push(l),
									u.directiveStart++,
									u.directiveEnd++,
									o && (u.providerIndexes += 1048576),
									n.push(d),
									s.push(d);
							}
							!o && r && g && n[p].componentProviders++;
						}
					}
				}
				function ct(e, t, n, r) {
					let o = i6(t),
						i = !!t.useClass;
					if (o || i) {
						let s = i ? M(t.useClass) : t,
							l = s.prototype,
							a = l.ngOnDestroy;
						if (a) {
							let i = e.destroyHooks || (e.destroyHooks = []);
							if (!o && t.multi) {
								ngDevMode &&
									Z(
										r,
										"indexInFactory when registering multi factory destroy hook",
									);
								let e = i.indexOf(n);
								-1 === e ? i.push(n, [r, a]) : i[e + 1].push(r, a);
							} else i.push(n, a);
						}
					}
				}
				function cn(e, t, n) {
					return n && e.componentProviders++, e.multi.push(t) - 1;
				}
				function cr(e, t, n, r) {
					for (let o = n; o < r; o++) if (t[o] === e) return o;
					return -1;
				}
				function co(e, t, n, r) {
					return cs(this.multi, []);
				}
				function ci(e, t, n, r) {
					let o;
					let i = this.multi;
					if (this.providerFactory) {
						let e = this.providerFactory.componentProviders,
							t = n9(n, n[1], this.providerFactory.index, r);
						cs(i, (o = t.slice(0, e)));
						for (let n = e; n < t.length; n++) o.push(t[n]);
					} else cs(i, (o = []));
					return o;
				}
				function cs(e, t) {
					for (let n = 0; n < e.length; n++) {
						let r = e[n];
						t.push(r());
					}
					return t;
				}
				function cl(e, t = []) {
					return (n) => {
						n.providersResolver = (n, r) =>
							(function (e, t, n) {
								let r = ne();
								if (r.firstCreatePass) {
									let o = td(e);
									ce(n, r.data, r.blueprint, o, !0),
										ce(t, r.data, r.blueprint, o, !1);
								}
							})(n, r ? r(e) : e, t);
					};
				}
				class ca {}
				class cu {}
				function cd(e, t) {
					return new cc(e, null != t ? t : null, []);
				}
				class cc extends ca {
					get injector() {
						return this._r3Injector;
					}
					destroy() {
						ngDevMode && Z(this.destroyCbs, "NgModule already destroyed");
						let e = this._r3Injector;
						e.destroyed || e.destroy(),
							this.destroyCbs.forEach((e) => e()),
							(this.destroyCbs = null);
					}
					onDestroy(e) {
						ngDevMode && Z(this.destroyCbs, "NgModule already destroyed"),
							this.destroyCbs.push(e);
					}
					constructor(e, t, n) {
						super(),
							(this._parent = t),
							(this._bootstrapComponents = []),
							(this.destroyCbs = []),
							(this.componentFactoryResolver = new lB(this));
						let r = te(e);
						ngDevMode &&
							Z(r, `NgModule '${j(e)}' is not a subtype of 'NgModuleType'.`),
							(this._bootstrapComponents = sB(r.bootstrap)),
							(this._r3Injector = s2(
								e,
								t,
								[
									{ provide: ca, useValue: this },
									{ provide: sj, useValue: this.componentFactoryResolver },
									...n,
								],
								j(e),
								new Set(["environment"]),
							)),
							this._r3Injector.resolveInjectorInitializers(),
							(this.instance = this._r3Injector.get(e));
					}
				}
				class cf extends cu {
					create(e) {
						return new cc(this.moduleType, e, []);
					}
					constructor(e) {
						super(), (this.moduleType = e);
					}
				}
				class ch extends ca {
					destroy() {
						this.injector.destroy();
					}
					onDestroy(e) {
						this.injector.onDestroy(e);
					}
					constructor(e) {
						super(),
							(this.componentFactoryResolver = new lB(this)),
							(this.instance = null);
						let t = new sn(
							[
								...e.providers,
								{ provide: ca, useValue: this },
								{ provide: sj, useValue: this.componentFactoryResolver },
							],
							e.parent || se(),
							e.debugName,
							new Set(["environment"]),
						);
						(this.injector = t),
							e.runEnvironmentInitializers && t.resolveInjectorInitializers();
					}
				}
				function cp(e, t, n = null) {
					let r = new ch({
						providers: e,
						parent: t,
						debugName: n,
						runEnvironmentInitializers: !0,
					});
					return r.injector;
				}
				let cm = (() => {
					class e {
						getOrCreateStandaloneInjector(e) {
							if (!e.standalone) return null;
							if (!this.cachedInjectors.has(e.id)) {
								let t = iJ(!1, e.type),
									n =
										t.length > 0
											? cp([t], this._injector, `Standalone[${e.type.name}]`)
											: null;
								this.cachedInjectors.set(e.id, n);
							}
							return this.cachedInjectors.get(e.id);
						}
						ngOnDestroy() {
							try {
								for (let e of this.cachedInjectors.values())
									null !== e && e.destroy();
							} finally {
								this.cachedInjectors.clear();
							}
						}
						constructor(e) {
							(this._injector = e), (this.cachedInjectors = new Map());
						}
					}
					return (
						(e.ɵprov = J({
							token: e,
							providedIn: "environment",
							factory: () => new e(e_(st)),
						})),
						e
					);
				})();
				function cg(e) {
					e.getStandaloneInjector = (t) =>
						t.get(cm).getOrCreateStandaloneInjector(e);
				}
				function cv(e) {
					ngDevMode && cS(e);
					let t = ol(e);
					if (null === t) return null;
					if (void 0 === t.component) {
						let e = t.lView;
						if (null === e) return null;
						t.component = (function (e, t) {
							let n = t[1].data[e],
								{ directiveStart: r, componentOffset: o } = n;
							return o > -1 ? t[r + o] : null;
						})(t.nodeIndex, e);
					}
					return t.component;
				}
				function cy(e) {
					cS(e);
					let t = ol(e),
						n = t ? t.lView : null;
					return null === n ? null : n[8];
				}
				function cb(e) {
					let t;
					let n = ol(e),
						r = n ? n.lView : null;
					if (null === r) return null;
					for (; 2 === r[1].type && (t = ob(r)); ) r = t;
					return 512 & r[2] ? null : r[8];
				}
				function c_(e) {
					let t = oh(e);
					return null !== t
						? [
								(function (e) {
									let t = (function (e) {
										ngDevMode && Z(e, "component");
										let t = ti(e) ? e : oh(e);
										for (; t && !(512 & t[2]); ) t = ob(t);
										return ngDevMode && ty(t), t;
									})(e);
									return (
										ngDevMode &&
											Z(
												t[8],
												"Root view has no context. Perhaps it is disconnected?",
											),
										t[8]
									);
								})(t),
						  ]
						: [];
				}
				function cj(e) {
					let t = ol(e),
						n = t ? t.lView : null;
					if (null === n) return s3.NULL;
					let r = n[1].data[t.nodeIndex];
					return new rn(r, n);
				}
				function cx(e) {
					if (e instanceof Text) return [];
					let t = ol(e),
						n = t ? t.lView : null;
					if (null === n) return [];
					let r = n[1],
						o = t.nodeIndex;
					return (null == r ? void 0 : r.data[o])
						? (void 0 === t.directives && (t.directives = ov(o, n)),
						  null === t.directives ? [] : [...t.directives])
						: [];
				}
				function cD(e) {
					let { constructor: t } = e;
					if (!t) throw Error("Unable to find the instance constructor");
					let n = e6(t);
					if (n)
						return {
							inputs: n.inputs,
							outputs: n.outputs,
							encapsulation: n.encapsulation,
							changeDetection: n.onPush ? eS.OnPush : eS.Default,
						};
					let r = e8(t);
					return r ? { inputs: r.inputs, outputs: r.outputs } : null;
				}
				function cw(e) {
					return ol(e).native;
				}
				function cM(e) {
					ngDevMode && cS(e);
					let t = ol(e),
						n = null === t ? null : t.lView;
					if (null === n) return [];
					let r = n[1],
						o = n[7],
						i = r.cleanup,
						s = [];
					if (i && o)
						for (let t = 0; t < i.length; ) {
							let r = i[t++],
								l = i[t++];
							if ("string" == typeof r) {
								let a = tW(n[l]),
									u = o[i[t++]],
									d = i[t++],
									c = "boolean" == typeof d || d >= 0 ? "dom" : "output",
									f = "boolean" == typeof d && d;
								e == a &&
									s.push({
										element: e,
										name: r,
										callback: u,
										useCapture: f,
										type: c,
									});
							}
						}
					return s.sort(cC), s;
				}
				function cC(e, t) {
					return e.name == t.name ? 0 : e.name < t.name ? -1 : 1;
				}
				function cS(e) {
					if ("undefined" != typeof Element && !(e instanceof Element))
						throw Error("Expecting instance of DOM Element");
				}
				function cE(e, t, n, r) {
					return eC(() => {
						null !== t &&
							(e.hasOwnProperty("decorators") && void 0 !== e.decorators
								? e.decorators.push(...t)
								: (e.decorators = t)),
							null !== n && (e.ctorParameters = n),
							null !== r &&
								(e.hasOwnProperty("propDecorators") &&
								void 0 !== e.propDecorators
									? (e.propDecorators = m._({}, e.propDecorators, r))
									: (e.propDecorators = r));
					});
				}
				function cO(e, t, n) {
					var r, o, i;
					let s = nc() + e,
						l = t9();
					return l[s] === sZ
						? ((r = l), (o = s), (i = n ? t.call(n) : t()), (r[o] = i))
						: l4(l, s);
				}
				function cA(e, t, n, r) {
					return cV(t9(), nc(), e, t, n, r);
				}
				function cI(e, t, n, r, o) {
					return cB(t9(), nc(), e, t, n, r, o);
				}
				function cP(e, t, n, r, o, i) {
					return cU(t9(), nc(), e, t, n, r, o, i);
				}
				function cT(e, t, n, r, o, i, s) {
					return cH(t9(), nc(), e, t, n, r, o, i, s);
				}
				function ck(e, t, n, r, o, i, s, l) {
					var a, u, d;
					let c = nc() + e,
						f = t9(),
						h = l9(f, c, n, r, o, i);
					return l6(f, c + 4, s) || h
						? ((a = f),
						  (u = c + 5),
						  (d = l ? t.call(l, n, r, o, i, s) : t(n, r, o, i, s)),
						  (a[u] = d))
						: l4(f, c + 5);
				}
				function cF(e, t, n, r, o, i, s, l, a) {
					var u, d, c;
					let f = nc() + e,
						h = t9(),
						p = l9(h, f, n, r, o, i);
					return l8(h, f + 4, s, l) || p
						? ((u = h),
						  (d = f + 6),
						  (c = a ? t.call(a, n, r, o, i, s, l) : t(n, r, o, i, s, l)),
						  (u[d] = c))
						: l4(h, f + 6);
				}
				function cR(e, t, n, r, o, i, s, l, a, u) {
					var d, c, f;
					let h = nc() + e,
						p = t9(),
						m = l9(p, h, n, r, o, i);
					return l7(p, h + 4, s, l, a) || m
						? ((d = p),
						  (c = h + 7),
						  (f = u ? t.call(u, n, r, o, i, s, l, a) : t(n, r, o, i, s, l, a)),
						  (d[c] = f))
						: l4(p, h + 7);
				}
				function cN(e, t, n, r, o, i, s, l, a, u, d) {
					var c, f, h;
					let p = nc() + e,
						m = t9(),
						g = l9(m, p, n, r, o, i);
					return l9(m, p + 4, s, l, a, u) || g
						? ((c = m),
						  (f = p + 8),
						  (h = d
								? t.call(d, n, r, o, i, s, l, a, u)
								: t(n, r, o, i, s, l, a, u)),
						  (c[f] = h))
						: l4(m, p + 8);
				}
				function cL(e, t, n, r) {
					return cz(t9(), nc(), e, t, n, r);
				}
				function c$(e, t) {
					ngDevMode && K(e, t);
					let n = e[t];
					return n === sZ ? void 0 : n;
				}
				function cV(e, t, n, r, o, i) {
					var s, l, a;
					let u = t + n;
					return l6(e, u, o)
						? ((s = e), (l = u + 1), (a = i ? r.call(i, o) : r(o)), (s[l] = a))
						: c$(e, u + 1);
				}
				function cB(e, t, n, r, o, i, s) {
					var l, a, u;
					let d = t + n;
					return l8(e, d, o, i)
						? ((l = e),
						  (a = d + 2),
						  (u = s ? r.call(s, o, i) : r(o, i)),
						  (l[a] = u))
						: c$(e, d + 2);
				}
				function cU(e, t, n, r, o, i, s, l) {
					var a, u, d;
					let c = t + n;
					return l7(e, c, o, i, s)
						? ((a = e),
						  (u = c + 3),
						  (d = l ? r.call(l, o, i, s) : r(o, i, s)),
						  (a[u] = d))
						: c$(e, c + 3);
				}
				function cH(e, t, n, r, o, i, s, l, a) {
					var u, d, c;
					let f = t + n;
					return l9(e, f, o, i, s, l)
						? ((u = e),
						  (d = f + 4),
						  (c = a ? r.call(a, o, i, s, l) : r(o, i, s, l)),
						  (u[d] = c))
						: c$(e, f + 4);
				}
				function cz(e, t, n, r, o, i) {
					var s, l, a;
					let u = t + n,
						d = !1;
					for (let t = 0; t < o.length; t++) l6(e, u++, o[t]) && (d = !0);
					return d
						? ((s = e), (l = u), (a = r.apply(i, o)), (s[l] = a))
						: c$(e, u);
				}
				function cW(e, t) {
					var n, r, o, i, s;
					let l;
					let a = ne(),
						u = e + 25;
					if (a.firstCreatePass) {
						if (
							((l = (function (e, t) {
								if (t)
									for (let n = t.length - 1; n >= 0; n--) {
										let r = t[n];
										if (e === r.name) return r;
									}
								if (ngDevMode)
									throw new O(
										-302,
										(function (e) {
											let t = t9(),
												n = t[15],
												r = n[8],
												o = r4(t),
												i = r
													? ` in the '${r.constructor.name}' component`
													: "",
												s = `Verify that it is ${
													o
														? "included in the '@Component.imports' of this component"
														: "declared or imported in this module"
												}`,
												l = `The pipe '${e}' could not be found${i}. ${s}`;
											return l;
										})(e),
									);
							})(t, a.pipeRegistry)),
							(a.data[u] = l),
							l.onDestroy)
						) {
							(null !== (n = a.destroyHooks) && void 0 !== n
								? n
								: (a.destroyHooks = [])
							).push(u, l.onDestroy);
						}
					} else l = a.data[u];
					let d = l.factory || (l.factory = tM(l.type, !0)),
						c = eu(s4);
					try {
						let e = nY(!1),
							t = d();
						return (
							nY(e),
							(r = a),
							(o = t9()),
							(i = u),
							(s = t),
							i >= r.data.length &&
								((r.data[i] = null), (r.blueprint[i] = null)),
							(o[i] = s),
							t
						);
					} finally {
						eu(c);
					}
				}
				function cq(e, t, n) {
					let r = e + 25,
						o = t9(),
						i = tY(o, r);
					return cK(o, r) ? cV(o, nc(), t, i.transform, n, i) : i.transform(n);
				}
				function cG(e, t, n, r) {
					let o = e + 25,
						i = t9(),
						s = tY(i, o);
					return cK(i, o)
						? cB(i, nc(), t, s.transform, n, r, s)
						: s.transform(n, r);
				}
				function cZ(e, t, n, r, o) {
					let i = e + 25,
						s = t9(),
						l = tY(s, i);
					return cK(s, i)
						? cU(s, nc(), t, l.transform, n, r, o, l)
						: l.transform(n, r, o);
				}
				function cY(e, t, n, r, o, i) {
					let s = e + 25,
						l = t9(),
						a = tY(l, s);
					return cK(l, s)
						? cH(l, nc(), t, a.transform, n, r, o, i, a)
						: a.transform(n, r, o, i);
				}
				function cQ(e, t, n) {
					let r = e + 25,
						o = t9(),
						i = tY(o, r);
					return cK(o, r)
						? cz(o, nc(), t, i.transform, n, i)
						: i.transform.apply(i, n);
				}
				function cK(e, t) {
					return e[1].data[t].pure;
				}
				class cJ extends v.Subject {
					emit(e) {
						super.next(e);
					}
					subscribe(e, t, n) {
						let r = e,
							o = t || (() => null),
							i = n;
						if (e && "object" == typeof e) {
							var s, l, a;
							(r = null === (s = e.next) || void 0 === s ? void 0 : s.bind(e)),
								(o =
									null === (l = e.error) || void 0 === l ? void 0 : l.bind(e)),
								(i =
									null === (a = e.complete) || void 0 === a
										? void 0
										: a.bind(e));
						}
						this.__isAsync && ((o = cX(o)), r && (r = cX(r)), i && (i = cX(i)));
						let u = super.subscribe({ next: r, error: o, complete: i });
						return e instanceof v.Subscription && e.add(u), u;
					}
					constructor(e = !1) {
						super(), (this.__isAsync = e);
					}
				}
				function cX(e) {
					return (t) => {
						setTimeout(e, void 0, t);
					};
				}
				let c0 = cJ;
				function c1() {
					return this._results[Symbol.iterator]();
				}
				class c5 {
					get changes() {
						return this._changes || (this._changes = new c0());
					}
					get(e) {
						return this._results[e];
					}
					map(e) {
						return this._results.map(e);
					}
					filter(e) {
						return this._results.filter(e);
					}
					find(e) {
						return this._results.find(e);
					}
					reduce(e, t) {
						return this._results.reduce(e, t);
					}
					forEach(e) {
						this._results.forEach(e);
					}
					some(e) {
						return this._results.some(e);
					}
					toArray() {
						return this._results.slice();
					}
					toString() {
						return this._results.toString();
					}
					reset(e, t) {
						this.dirty = !1;
						let n = rD(e);
						(this._changesDetected = !(function (e, t, n) {
							if (e.length !== t.length) return !1;
							for (let r = 0; r < e.length; r++) {
								let o = e[r],
									i = t[r];
								if ((n && ((o = n(o)), (i = n(i))), i !== o)) return !1;
							}
							return !0;
						})(this._results, n, t)) &&
							((this._results = n),
							(this.length = n.length),
							(this.last = n[this.length - 1]),
							(this.first = n[0]));
					}
					notifyOnChanges() {
						this._changes &&
							(this._changesDetected || !this._emitDistinctChangesOnly) &&
							this._changes.emit(this);
					}
					setDirty() {
						this.dirty = !0;
					}
					destroy() {
						this.changes.complete(), this.changes.unsubscribe();
					}
					constructor(e = !1) {
						(this._emitDistinctChangesOnly = e),
							(this.dirty = !0),
							(this._results = []),
							(this._changesDetected = !1),
							(this._changes = null),
							(this.length = 0),
							(this.first = void 0),
							(this.last = void 0);
						let t = c5.prototype;
						!t[Symbol.iterator] && (t[Symbol.iterator] = c1);
					}
				}
				let c2 = (() => {
						class e {}
						return (e.__NG_ELEMENT_ID__ = c6), e;
					})(),
					c3 = c2,
					c4 = class extends c3 {
						get ssrId() {
							var e;
							return (
								(null === (e = this._declarationTContainer.tView) ||
								void 0 === e
									? void 0
									: e.ssrId) || null
							);
						}
						createEmbeddedView(e, t) {
							return this.createEmbeddedViewImpl(e, t, null);
						}
						createEmbeddedViewImpl(e, t, n) {
							let r = this._declarationTContainer.tView,
								o = s8(
									this._declarationLView,
									r,
									e,
									16,
									null,
									r.declTNode,
									null,
									null,
									null,
									t || null,
									n || null,
								),
								i = this._declarationLView[this._declarationTContainer.index];
							ngDevMode && tg(i), (o[16] = i);
							let s = this._declarationLView[18];
							return (
								null !== s && (o[18] = s.createEmbeddedView(r)),
								lA(r, o, e),
								new l$(o)
							);
						}
						constructor(e, t, n) {
							super(),
								(this._declarationLView = e),
								(this._declarationTContainer = t),
								(this.elementRef = n);
						}
					};
				function c6() {
					return c8(nr(), t9());
				}
				function c8(e, t) {
					return 4 & e.type
						? (ngDevMode && Z(e.tView, "TView must be allocated"),
						  new c4(t, e, sD(e, t)))
						: null;
				}
				let c7 = (e, t) => null;
				function c9(e, t) {
					return c7(e, t);
				}
				let fe = (() => {
					class e {}
					return (e.__NG_ELEMENT_ID__ = ft), e;
				})();
				function ft() {
					let e = nr();
					return fs(e, t9());
				}
				let fn = fe,
					fr = class extends fn {
						get element() {
							return sD(this._hostTNode, this._hostLView);
						}
						get injector() {
							return new rn(this._hostTNode, this._hostLView);
						}
						get parentInjector() {
							let e = n5(this._hostTNode, this._hostLView);
							if (-1 === e) return new rn(null, this._hostLView);
							{
								let t = nG(e, this._hostLView),
									n = nq(e);
								ngDevMode && tw(t, n);
								let r = t[1].data[n + 8];
								return new rn(r, t);
							}
						}
						clear() {
							for (; this.length > 0; ) this.remove(this.length - 1);
						}
						get(e) {
							let t = (function (e) {
								return e[8];
							})(this._lContainer);
							return (null !== t && t[e]) || null;
						}
						get length() {
							return this._lContainer.length - 11;
						}
						createEmbeddedView(e, t, n) {
							var r;
							let o, i;
							"number" == typeof n
								? (o = n)
								: null != n && ((o = n.index), (i = n.injector));
							let s = ((r = this._lContainer), c7(r, e.ssrId)),
								l = e.createEmbeddedViewImpl(t || {}, i, s);
							return this.insertImpl(l, o, !!s), l;
						}
						createComponent(e, t, n, r, o) {
							var i, s, l, a;
							let u;
							let d = e && !rx(e);
							if (d)
								ngDevMode &&
									V(
										"object" != typeof t,
										!0,
										"It looks like Component factory was provided as the first argument and an options object as the second argument. This combination of arguments is incompatible. You can either change the first argument to provide Component type or change the second argument to be a number (representing an index at which to insert the new component's host view into this container)",
									),
									(u = t);
							else {
								ngDevMode &&
									(Z(
										e6(e),
										"Provided Component class doesn't contain Component definition. Please check whether provided class has @Component decorator.",
									),
									V(
										"number" != typeof t,
										!0,
										"It looks like Component type was provided as the first argument and a number (representing an index at which to insert the new component's host view into this container as the second argument. This combination of arguments is incompatible. Please use an object as the second argument instead.",
									));
								let i = t || {};
								ngDevMode &&
									i.environmentInjector &&
									i.ngModuleRef &&
									Y(
										"Cannot pass both environmentInjector and ngModuleRef options to createComponent().",
									),
									(u = i.index),
									(n = i.injector),
									(r = i.projectableNodes),
									(o = i.environmentInjector || i.ngModuleRef);
							}
							let c = d ? e : new lz(e6(e)),
								f = n || this.parentInjector;
							if (!o && null == c.ngModule) {
								let e = d ? f : this.parentInjector,
									t = e.get(st, null);
								t && (o = t);
							}
							let h = e6(
								null !== (i = c.componentType) && void 0 !== i ? i : {},
							);
							let p =
									((a = this._lContainer),
									c7(
										a,
										null !== (s = null == h ? void 0 : h.id) && void 0 !== s
											? s
											: null,
									)),
								m =
									null !== (l = null == p ? void 0 : p.firstChild) &&
									void 0 !== l
										? l
										: null,
								g = c.create(f, r, m, o);
							return this.insertImpl(g.hostView, u, !!p), g;
						}
						insert(e, t) {
							return this.insertImpl(e, t, !1);
						}
						insertImpl(e, t, n) {
							let r = e._lView,
								o = r[1];
							if (ngDevMode && e.destroyed)
								throw Error(
									"Cannot insert a destroyed View in a ViewContainer!",
								);
							if (ts(r[3])) {
								let t = this.indexOf(e);
								if (-1 !== t) this.detach(t);
								else {
									let t = r[3];
									ngDevMode &&
										V(
											ts(t),
											!0,
											"An attached view should have its PARENT point to a container.",
										);
									let n = new fr(t, t[6], t[3]);
									n.detach(n.indexOf(e));
								}
							}
							let i = this._adjustIndex(t),
								s = this._lContainer;
							if (
								(!(function (e, t, n, r) {
									ngDevMode && ty(t), ngDevMode && tg(n);
									let o = 11 + r,
										i = n.length;
									r > 0 && (n[o - 1][4] = t),
										r < i - 11
											? ((t[4] = n[o]), rM(n, 11 + r, t))
											: (n.push(t), (t[4] = null)),
										(t[3] = n);
									let s = t[16];
									null !== s &&
										n !== s &&
										(function (e, t) {
											ngDevMode && Z(t, "LView required"), ngDevMode && tg(e);
											let n = e[9],
												r = t[3];
											ngDevMode && tg(r);
											let o = r[3][15];
											ngDevMode && Z(o, "Missing insertedComponentLView");
											let i = t[15];
											ngDevMode && Z(i, "Missing declaredComponentLView"),
												i !== o && (e[2] = !0),
												null === n ? (e[9] = [t]) : n.push(t);
										})(s, t);
									let l = t[18];
									null !== l && l.insertView(e), (t[2] |= 128);
								})(o, r, s, i),
								!n)
							) {
								let e = oz(i, s),
									t = r[11],
									n = oN(t, s[7]);
								if (null !== n) {
									var l, a, u, d, c, f;
									(l = o),
										(a = s[6]),
										(u = t),
										(d = r),
										(c = n),
										(f = e),
										(d[0] = c),
										(d[6] = a),
										oG(l, d, u, 1, c, f);
								}
							}
							return e.attachToViewContainerRef(), rM(fi(s), i, e), e;
						}
						move(e, t) {
							if (ngDevMode && e.destroyed)
								throw Error("Cannot move a destroyed View in a ViewContainer!");
							return this.insert(e, t);
						}
						indexOf(e) {
							let t = (function (e) {
								return e[8];
							})(this._lContainer);
							return null !== t ? t.indexOf(e) : -1;
						}
						remove(e) {
							let t = this._adjustIndex(e, -1),
								n = oO(this._lContainer, t);
							n && (rC(fi(this._lContainer), t), oA(n[1], n));
						}
						detach(e) {
							let t = this._adjustIndex(e, -1),
								n = oO(this._lContainer, t),
								r = n && null != rC(fi(this._lContainer), t);
							return r ? new l$(n) : null;
						}
						_adjustIndex(e, t = 0) {
							return null == e
								? this.length + t
								: (ngDevMode &&
										(q(e, -1, `ViewRef index must be positive, got ${e}`),
										z(e, this.length + 1 + t, "index")),
								  e);
						}
						constructor(e, t, n) {
							super(),
								(this._lContainer = e),
								(this._hostTNode = t),
								(this._hostLView = n);
						}
					};
				function fo(e) {
					return e[8];
				}
				function fi(e) {
					return e[8] || (e[8] = []);
				}
				function fs(e, t) {
					let n;
					ngDevMode && nz(e, 15);
					let r = t[e.index];
					return (
						ts(r)
							? (n = r)
							: ((n = lb(r, t, null, e)), (t[e.index] = n), lj(t, n)),
						fl(n, t, e, r),
						new fr(n, e, t)
					);
				}
				let fl = function (e, t, n, r) {
					let o;
					!e[7] &&
						((o =
							8 & n.type
								? tW(r)
								: (function (e, t) {
										var n, r;
										let o = e[11];
										ngDevMode && ngDevMode.rendererCreateComment++;
										let i = o.createComment(ngDevMode ? "container" : ""),
											s = tG(t, e),
											l = oN(o, s);
										return (
											ok(o, l, i, ((n = o), (r = s), n.nextSibling(r)), !1), i
										);
								  })(t, n)),
						(e[7] = o));
				};
				class fa {
					clone() {
						return new fa(this.queryList);
					}
					setDirty() {
						this.queryList.setDirty();
					}
					constructor(e) {
						(this.queryList = e), (this.matches = null);
					}
				}
				class fu {
					createEmbeddedView(e) {
						let t = e.queries;
						if (null !== t) {
							let n =
									null !== e.contentQueries ? e.contentQueries[0] : t.length,
								r = [];
							for (let e = 0; e < n; e++) {
								let n = t.getByIndex(e),
									o = this.queries[n.indexInDeclarationView];
								r.push(o.clone());
							}
							return new fu(r);
						}
						return null;
					}
					insertView(e) {
						this.dirtyQueriesWithMatches(e);
					}
					detachView(e) {
						this.dirtyQueriesWithMatches(e);
					}
					dirtyQueriesWithMatches(e) {
						for (let t = 0; t < this.queries.length; t++)
							null !== f_(e, t).matches && this.queries[t].setDirty();
					}
					constructor(e = []) {
						this.queries = e;
					}
				}
				class fd {
					constructor(e, t, n = null) {
						(this.predicate = e), (this.flags = t), (this.read = n);
					}
				}
				class fc {
					elementStart(e, t) {
						ngDevMode &&
							tb(
								e,
								"Queries should collect results on the first template pass only",
							);
						for (let n = 0; n < this.queries.length; n++)
							this.queries[n].elementStart(e, t);
					}
					elementEnd(e) {
						for (let t = 0; t < this.queries.length; t++)
							this.queries[t].elementEnd(e);
					}
					embeddedTView(e) {
						let t = null;
						for (let n = 0; n < this.length; n++) {
							let r = null !== t ? t.length : 0,
								o = this.getByIndex(n).embeddedTView(e, r);
							o &&
								((o.indexInDeclarationView = n),
								null !== t ? t.push(o) : (t = [o]));
						}
						return null !== t ? new fc(t) : null;
					}
					template(e, t) {
						ngDevMode &&
							tb(
								e,
								"Queries should collect results on the first template pass only",
							);
						for (let n = 0; n < this.queries.length; n++)
							this.queries[n].template(e, t);
					}
					getByIndex(e) {
						return ngDevMode && K(this.queries, e), this.queries[e];
					}
					get length() {
						return this.queries.length;
					}
					track(e) {
						this.queries.push(e);
					}
					constructor(e = []) {
						this.queries = e;
					}
				}
				class ff {
					elementStart(e, t) {
						this.isApplyingToNode(t) && this.matchTNode(e, t);
					}
					elementEnd(e) {
						this._declarationNodeIndex === e.index &&
							(this._appliesToNextNode = !1);
					}
					template(e, t) {
						this.elementStart(e, t);
					}
					embeddedTView(e, t) {
						return this.isApplyingToNode(e)
							? ((this.crossesNgTemplate = !0),
							  this.addMatch(-e.index, t),
							  new ff(this.metadata))
							: null;
					}
					isApplyingToNode(e) {
						if (this._appliesToNextNode && (1 & this.metadata.flags) != 1) {
							let t = this._declarationNodeIndex,
								n = e.parent;
							for (; null !== n && 8 & n.type && n.index !== t; ) n = n.parent;
							return t === (null !== n ? n.index : -1);
						}
						return this._appliesToNextNode;
					}
					matchTNode(e, t) {
						let n = this.metadata.predicate;
						if (Array.isArray(n))
							for (let r = 0; r < n.length; r++) {
								let o = n[r];
								this.matchTNodeWithReadOption(
									e,
									t,
									(function (e, t) {
										let n = e.localNames;
										if (null !== n) {
											for (let e = 0; e < n.length; e += 2)
												if (n[e] === t) return n[e + 1];
										}
										return null;
									})(t, o),
								),
									this.matchTNodeWithReadOption(e, t, n7(t, e, o, !1, !1));
							}
						else
							n === c2
								? 4 & t.type && this.matchTNodeWithReadOption(e, t, -1)
								: this.matchTNodeWithReadOption(e, t, n7(t, e, n, !1, !1));
					}
					matchTNodeWithReadOption(e, t, n) {
						if (null !== n) {
							let r = this.metadata.read;
							if (null !== r) {
								if (r === sw || r === fe || (r === c2 && 4 & t.type))
									this.addMatch(t.index, -2);
								else {
									let n = n7(t, e, r, !1, !1);
									null !== n && this.addMatch(t.index, n);
								}
							} else this.addMatch(t.index, n);
						}
					}
					addMatch(e, t) {
						null === this.matches
							? (this.matches = [e, t])
							: this.matches.push(e, t);
					}
					constructor(e, t = -1) {
						(this.metadata = e),
							(this.matches = null),
							(this.indexInDeclarationView = -1),
							(this.crossesNgTemplate = !1),
							(this._appliesToNextNode = !0),
							(this._declarationNodeIndex = t);
					}
				}
				function fh(e, t, n, r) {
					let o = t[18].queries[r];
					if (null === o.matches) {
						let r = e.data,
							i = n.matches,
							s = [];
						for (let e = 0; e < i.length; e += 2) {
							let o = i[e];
							if (o < 0) s.push(null);
							else {
								ngDevMode && K(r, o);
								let l = r[o];
								s.push(
									(function (e, t, n, r) {
										if (-1 === n) {
											var o, i;
											return (
												(o = t),
												(i = e),
												11 & o.type ? sD(o, i) : 4 & o.type ? c8(o, i) : null
											);
										}
										if (-2 === n)
											return (function (e, t, n) {
												if (n === sw) return sD(t, e);
												if (n === c2) return c8(t, e);
												if (n === fe) return ngDevMode && nz(t, 15), fs(t, e);
												else
													ngDevMode &&
														Y(
															`Special token to read should be one of ElementRef, TemplateRef or ViewContainerRef but got ${j(
																n,
															)}.`,
														);
											})(e, t, r);
										return n9(e, e[1], n, t);
									})(t, l, i[e + 1], n.metadata.read),
								);
							}
						}
						o.matches = s;
					}
					return o.matches;
				}
				function fp(e) {
					let t = t9(),
						n = ne(),
						r = nb();
					n_(r + 1);
					let o = f_(n, r);
					if (e.dirty && tK(t) === ((2 & o.metadata.flags) == 2)) {
						if (null === o.matches) e.reset([]);
						else {
							let i = o.crossesNgTemplate
								? (function e(t, n, r, o) {
										let i = t.queries.getByIndex(r),
											s = i.matches;
										if (null !== s) {
											let l = fh(t, n, i, r);
											for (let t = 0; t < s.length; t += 2) {
												let r = s[t];
												if (r > 0) o.push(l[t / 2]);
												else {
													let i = s[t + 1],
														l = n[-r];
													ngDevMode && tg(l);
													for (let t = 11; t < l.length; t++) {
														let n = l[t];
														n[16] === n[3] && e(n[1], n, i, o);
													}
													if (null !== l[9]) {
														let t = l[9];
														for (let n = 0; n < t.length; n++) {
															let r = t[n];
															e(r[1], r, i, o);
														}
													}
												}
											}
										}
										return o;
								  })(n, t, r, [])
								: fh(n, t, o, r);
							e.reset(i, sM), e.notifyOnChanges();
						}
						return !0;
					}
					return !1;
				}
				function fm(e, t, n) {
					ngDevMode && N(t, "Expecting flags");
					let r = ne();
					r.firstCreatePass &&
						(fb(r, new fd(e, t, n), -1),
						(2 & t) == 2 && (r.staticViewQueries = !0)),
						fy(r, t9(), t);
				}
				function fg(e, t, n, r) {
					ngDevMode && N(n, "Expecting flags");
					let o = ne();
					if (o.firstCreatePass) {
						let i = nr();
						fb(o, new fd(t, n, r), i.index),
							(function (e, t) {
								let n = e.contentQueries || (e.contentQueries = []),
									r = n.length ? n[n.length - 1] : -1;
								t !== r && n.push(e.queries.length - 1, t);
							})(o, e),
							(2 & n) == 2 && (o.staticContentQueries = !0);
					}
					fy(o, t9(), n);
				}
				function fv() {
					return (function (e, t) {
						return (
							ngDevMode &&
								Z(
									e[18],
									"LQueries should be defined when trying to load a query",
								),
							ngDevMode && K(e[18].queries, t),
							e[18].queries[t].queryList
						);
					})(t9(), nb());
				}
				function fy(e, t, n) {
					let r = new c5((4 & n) == 4);
					!(function (e, t, n, r) {
						let o = lw(t);
						ngDevMode &&
							Z(
								n,
								"Cleanup context is mandatory when registering framework-level destroy hooks",
							),
							o.push(n),
							e.firstCreatePass
								? lM(e).push(r, o.length - 1)
								: ngDevMode && Object.freeze(lM(e));
					})(e, t, r, r.destroy),
						null === t[18] && (t[18] = new fu()),
						t[18].queries.push(new fa(r));
				}
				function fb(e, t, n) {
					null === e.queries && (e.queries = new fc()),
						e.queries.track(new ff(t, n));
				}
				function f_(e, t) {
					return (
						ngDevMode &&
							Z(e.queries, "TQueries must be defined to retrieve a TQuery"),
						e.queries.getByIndex(t)
					);
				}
				function fj(e, t) {
					return c8(e, t);
				}
				let fx = {
						ɵɵattribute: ae,
						ɵɵattributeInterpolate1: ad,
						ɵɵattributeInterpolate2: ac,
						ɵɵattributeInterpolate3: af,
						ɵɵattributeInterpolate4: ah,
						ɵɵattributeInterpolate5: ap,
						ɵɵattributeInterpolate6: am,
						ɵɵattributeInterpolate7: ag,
						ɵɵattributeInterpolate8: av,
						ɵɵattributeInterpolateV: ay,
						ɵɵdefineComponent: eK,
						ɵɵdefineDirective: e3,
						ɵɵdefineInjectable: J,
						ɵɵdefineInjector: X,
						ɵɵdefineNgModule: e1,
						ɵɵdefinePipe: e4,
						ɵɵdirectiveInject: s4,
						ɵɵgetInheritedFactory: ro,
						ɵɵinject: e_,
						ɵɵinjectAttribute: rl,
						ɵɵinvalidFactory: s6,
						ɵɵinvalidFactoryDep: ej,
						ɵɵtemplateRefExtractor: fj,
						ɵɵresetView: nn,
						ɵɵHostDirectivesFeature: lX,
						ɵɵNgOnChangesFeature: tF,
						ɵɵProvidersFeature: cl,
						ɵɵCopyDefinitionFeature: lJ,
						ɵɵInheritDefinitionFeature: lZ,
						ɵɵStandaloneFeature: cg,
						ɵɵnextContext: aV,
						ɵɵnamespaceHTML: nP,
						ɵɵnamespaceMathML: nI,
						ɵɵnamespaceSVG: nA,
						ɵɵenableBindings: t8,
						ɵɵdisableBindings: t7,
						ɵɵelementStart: aw,
						ɵɵelementEnd: aM,
						ɵɵelement: aC,
						ɵɵelementContainerStart: aE,
						ɵɵelementContainerEnd: aO,
						ɵɵelementContainer: aA,
						ɵɵpureFunction0: cO,
						ɵɵpureFunction1: cA,
						ɵɵpureFunction2: cI,
						ɵɵpureFunction3: cP,
						ɵɵpureFunction4: cT,
						ɵɵpureFunction5: ck,
						ɵɵpureFunction6: cF,
						ɵɵpureFunction7: cR,
						ɵɵpureFunction8: cN,
						ɵɵpureFunctionV: cL,
						ɵɵgetCurrentView: aP,
						ɵɵrestoreView: nt,
						ɵɵlistener: aF,
						ɵɵprojection: aU,
						ɵɵsyntheticHostProperty: u9,
						ɵɵsyntheticHostListener: aR,
						ɵɵpipeBind1: cq,
						ɵɵpipeBind2: cG,
						ɵɵpipeBind3: cZ,
						ɵɵpipeBind4: cY,
						ɵɵpipeBindV: cQ,
						ɵɵprojectionDef: aB,
						ɵɵhostProperty: u7,
						ɵɵproperty: ax,
						ɵɵpropertyInterpolate: aH,
						ɵɵpropertyInterpolate1: az,
						ɵɵpropertyInterpolate2: aW,
						ɵɵpropertyInterpolate3: aq,
						ɵɵpropertyInterpolate4: aG,
						ɵɵpropertyInterpolate5: aZ,
						ɵɵpropertyInterpolate6: aY,
						ɵɵpropertyInterpolate7: aQ,
						ɵɵpropertyInterpolate8: aK,
						ɵɵpropertyInterpolateV: aJ,
						ɵɵpipe: cW,
						ɵɵqueryRefresh: fp,
						ɵɵviewQuery: fm,
						ɵɵloadQuery: fv,
						ɵɵcontentQuery: fg,
						ɵɵreference: aj,
						ɵɵclassMap: uu,
						ɵɵclassMapInterpolate1: uF,
						ɵɵclassMapInterpolate2: uR,
						ɵɵclassMapInterpolate3: uN,
						ɵɵclassMapInterpolate4: uL,
						ɵɵclassMapInterpolate5: u$,
						ɵɵclassMapInterpolate6: uV,
						ɵɵclassMapInterpolate7: uB,
						ɵɵclassMapInterpolate8: uU,
						ɵɵclassMapInterpolateV: uH,
						ɵɵstyleMap: ul,
						ɵɵstyleMapInterpolate1: uz,
						ɵɵstyleMapInterpolate2: uW,
						ɵɵstyleMapInterpolate3: uq,
						ɵɵstyleMapInterpolate4: uG,
						ɵɵstyleMapInterpolate5: uZ,
						ɵɵstyleMapInterpolate6: uY,
						ɵɵstyleMapInterpolate7: uQ,
						ɵɵstyleMapInterpolate8: uK,
						ɵɵstyleMapInterpolateV: uJ,
						ɵɵstyleProp: ui,
						ɵɵstylePropInterpolate1: uX,
						ɵɵstylePropInterpolate2: u0,
						ɵɵstylePropInterpolate3: u1,
						ɵɵstylePropInterpolate4: u5,
						ɵɵstylePropInterpolate5: u2,
						ɵɵstylePropInterpolate6: u3,
						ɵɵstylePropInterpolate7: u4,
						ɵɵstylePropInterpolate8: u6,
						ɵɵstylePropInterpolateV: u8,
						ɵɵclassProp: us,
						ɵɵadvance: sY,
						ɵɵtemplate: ab,
						ɵɵtext: uD,
						ɵɵtextInterpolate: uM,
						ɵɵtextInterpolate1: uC,
						ɵɵtextInterpolate2: uS,
						ɵɵtextInterpolate3: uE,
						ɵɵtextInterpolate4: uO,
						ɵɵtextInterpolate5: uA,
						ɵɵtextInterpolate6: uI,
						ɵɵtextInterpolate7: uP,
						ɵɵtextInterpolate8: uT,
						ɵɵtextInterpolateV: uk,
						ɵɵi18n: d4,
						ɵɵi18nAttributes: d6,
						ɵɵi18nExp: d8,
						ɵɵi18nStart: d2,
						ɵɵi18nEnd: d3,
						ɵɵi18nApply: d7,
						ɵɵi18nPostprocess: d9,
						ɵɵresolveWindow: sN,
						ɵɵresolveDocument: sL,
						ɵɵresolveBody: s$,
						ɵɵsetComponentScope: eJ,
						ɵɵsetNgModuleScope: e5,
						ɵɵregisterNgModuleType: rX,
						ɵɵsanitizeHtml: iN,
						ɵɵsanitizeStyle: iL,
						ɵɵsanitizeResourceUrl: iV,
						ɵɵsanitizeScript: iB,
						ɵɵsanitizeUrl: i$,
						ɵɵsanitizeUrlOrResourceUrl: iz,
						ɵɵtrustConstantHtml: iU,
						ɵɵtrustConstantResourceUrl: iH,
						ɵɵvalidateIframeAttribute: oX,
						forwardRef: w,
						resolveForwardRef: M,
					},
					fD = null;
				function fw(e) {
					return void 0 !== e.ngModule;
				}
				function fM(e) {
					return !!te(e);
				}
				let fC = [],
					fS = !1;
				function fE(e) {
					return Array.isArray(e) ? e.every(fE) : !!M(e);
				}
				function fO(e, t, n) {
					let r;
					if (fP.get(e) || e9(e)) return;
					if ((fP.set(e, !0), (e = M(e)), n)) {
						if (!(r = te(e)))
							throw Error(
								`Unexpected value '${e.name}' imported by the module '${n.name}'. Please add an @NgModule annotation.`,
							);
					} else r = te(e, !0);
					let o = [],
						i = sB(r.declarations),
						s = sB(r.imports);
					rD(s)
						.map(fA)
						.forEach((t) => {
							d(t, e), fO(t, !1, e);
						});
					let l = sB(r.exports);
					i.forEach(function (t) {
						t = M(t);
						let n = e6(t) || e8(t) || e7(t);
						!n &&
							o.push(
								`Unexpected value '${P(t)}' declared by the module '${P(
									e,
								)}'. Please add a @Pipe/@Directive/@Component annotation.`,
							);
					}),
						i.forEach(function (e) {
							e = M(e);
							let t = e8(e);
							!e6(e) &&
								t &&
								0 == t.selectors.length &&
								o.push(`Directive ${P(e)} has no selector, please add it!`);
						}),
						i.forEach((t) =>
							(function (e, t) {
								e = M(e);
								let n = e6(e) || e8(e) || e7(e);
								if (null == n ? void 0 : n.standalone) {
									let n = `"${P(t)}" NgModule`;
									o.push(
										(function (e, t) {
											let n = `Unexpected "${P(
													e,
												)}" found in the "declarations" array of the`,
												r = `"${P(
													e,
												)}" is marked as standalone and can't be declared in any NgModule - did you intend to import it instead (by adding it to the "imports" array)?`;
											return `${n} ${t}, ${r}`;
										})(e, n),
									);
								}
							})(t, e),
						);
					let a = [
						...i.map(M),
						...rD(
							s.map(function e(t) {
								t = M(t);
								let n = te(t);
								return null === n
									? [t]
									: [
											...rD(
												sB(n.exports).map((t) => {
													let n = te(t);
													return n ? (fO(t, !1), e(t)) : t;
												}),
											),
									  ];
							}),
						).map(M),
					];
					l.forEach(function (t) {
						t = M(t);
						let n =
							(e6(t) && "component") ||
							(e8(t) && "directive") ||
							(e7(t) && "pipe");
						n &&
							-1 === a.lastIndexOf(t) &&
							o.push(
								`Can't export ${n} ${P(t)} from ${P(
									e,
								)} as it was neither declared nor imported!`,
							);
					}),
						i.forEach((n) =>
							(function (t, n) {
								t = M(t);
								let r = fI.get(t);
								if (r && r !== e) {
									if (!n) {
										let n = [r, e].map(P).sort();
										o.push(
											`Type ${P(t)} is part of the declarations of 2 modules: ${
												n[0]
											} and ${n[1]}! Please consider moving ${P(
												t,
											)} to a higher module that imports ${n[0]} and ${
												n[1]
											}. You can also create a new NgModule that exports and includes ${P(
												t,
											)} then import that NgModule in ${n[0]} and ${n[1]}.`,
										);
									}
								} else fI.set(t, e);
							})(n, t),
						);
					let u = (function (e, t) {
						let n = null;
						return r(e.__annotations__), r(e.decorators), n;
						function r(e) {
							e && e.forEach(o);
						}
						function o(e) {
							if (!n) {
								let r = Object.getPrototypeOf(e);
								if (r.ngMetadataName == t) n = e;
								else if (e.type) {
									let r = Object.getPrototypeOf(e.type);
									r.ngMetadataName == t && (n = e.args[0]);
								}
							}
						}
					})(e, "NgModule");
					if (
						(u &&
							(u.imports &&
								rD(u.imports)
									.map(fA)
									.forEach((t) => {
										d(t, e), fO(t, !1, e);
									}),
							u.bootstrap &&
								rw(u.bootstrap, function (e) {
									!e6((e = M(e))) &&
										o.push(`${P(e)} cannot be used as an entry component.`),
										e9(e) &&
											o.push(
												`The \`${P(
													e,
												)}\` class is a standalone component, which can not be used in the \`@NgModule.bootstrap\` array. Use the \`bootstrapApplication\` function for bootstrap instead.`,
											);
								}),
							u.bootstrap &&
								rw(u.bootstrap, function (e) {
									e = M(e);
									let t = fI.get(e);
									!t &&
										!e9(e) &&
										o.push(
											`Component ${P(
												e,
											)} is not part of any NgModule or the module has not been imported into your module.`,
										);
								})),
						o.length)
					)
						throw Error(o.join("\n"));
					function d(e, t) {
						e = M(e);
						let n = e6(e) || e8(e);
						if (null !== n && !n.standalone)
							throw Error(
								`Unexpected directive '${e.name}' imported by the module '${t.name}'. Please add an @NgModule annotation.`,
							);
						let r = e7(e);
						if (null !== r && !r.standalone)
							throw Error(
								`Unexpected pipe '${e.name}' imported by the module '${t.name}'. Please add an @NgModule annotation.`,
							);
					}
				}
				function fA(e) {
					return (e = M(e)).ngModule || e;
				}
				let fI = new WeakMap(),
					fP = new WeakMap();
				function fT(e, t) {
					(e.directiveDefs = () =>
						Array.from(t.compilation.directives)
							.map((e) => (e.hasOwnProperty(eI) ? e6(e) : e8(e)))
							.filter((e) => !!e)),
						(e.pipeDefs = () =>
							Array.from(t.compilation.pipes).map((e) => e7(e))),
						(e.schemas = t.schemas),
						(e.tView = null);
				}
				function fk(e) {
					if (fM(e))
						return (function (e) {
							let t = te(e, !0);
							if (null !== t.transitiveCompileScopes)
								return t.transitiveCompileScopes;
							let n = {
								schemas: t.schemas || null,
								compilation: { directives: new Set(), pipes: new Set() },
								exported: { directives: new Set(), pipes: new Set() },
							};
							return (
								sB(t.imports).forEach((e) => {
									let t = fk(e);
									t.exported.directives.forEach((e) =>
										n.compilation.directives.add(e),
									),
										t.exported.pipes.forEach((e) => n.compilation.pipes.add(e));
								}),
								sB(t.declarations).forEach((e) => {
									e7(e)
										? n.compilation.pipes.add(e)
										: n.compilation.directives.add(e);
								}),
								sB(t.exports).forEach((e) => {
									if (fM(e)) {
										let t = fk(e);
										t.exported.directives.forEach((e) => {
											n.compilation.directives.add(e),
												n.exported.directives.add(e);
										}),
											t.exported.pipes.forEach((e) => {
												n.compilation.pipes.add(e), n.exported.pipes.add(e);
											});
									} else
										e7(e)
											? n.exported.pipes.add(e)
											: n.exported.directives.add(e);
								}),
								(t.transitiveCompileScopes = n),
								n
							);
						})(e);
					if (e9(e)) {
						let t = e6(e) || e8(e);
						if (null !== t)
							return {
								schemas: null,
								compilation: { directives: new Set(), pipes: new Set() },
								exported: { directives: new Set([e]), pipes: new Set() },
							};
						let n = e7(e);
						if (null !== n)
							return {
								schemas: null,
								compilation: { directives: new Set(), pipes: new Set() },
								exported: { directives: new Set(), pipes: new Set([e]) },
							};
					}
					throw Error(`${e.name} does not have a module def (ɵmod property)`);
				}
				function fF(e) {
					return fw(e) ? e.ngModule : e;
				}
				let fR = 0;
				function fN(e, t) {
					let n = null;
					f$(e, t || {}),
						Object.defineProperty(e, eP, {
							get: () => {
								if (null === n) {
									let r = fL(e, t || {}),
										o = r_({ usage: 0, kind: "directive", type: e });
									n = o.compileDirective(fx, r.sourceMapUrl, r.metadata);
								}
								return n;
							},
							configurable: !!ngDevMode,
						});
				}
				function fL(e, t) {
					let n = e && e.name,
						r = `ng:///${n}/ɵdir.js`,
						o = r_({ usage: 0, kind: "directive", type: e }),
						i = fV(e, t);
					return (
						(i.typeSourceSpan = o.createParseSourceSpan("Directive", n, r)),
						i.usesInheritance && fB(e),
						{ metadata: i, sourceMapUrl: r }
					);
				}
				function f$(e, t) {
					let n = null;
					Object.defineProperty(e, eF, {
						get: () => {
							if (null === n) {
								let r = fL(e, t),
									o = r_({ usage: 0, kind: "directive", type: e });
								n = o.compileFactory(fx, `ng:///${e.name}/ɵfac.js`, {
									name: r.metadata.name,
									type: r.metadata.type,
									typeArgumentCount: 0,
									deps: rq(e),
									target: o.FactoryTarget.Directive,
								});
							}
							return n;
						},
						configurable: !!ngDevMode,
					});
				}
				function fV(e, t) {
					var n;
					let r = rW(),
						o = r.ownPropMetadata(e);
					return {
						name: e.name,
						type: e,
						selector: void 0 !== t.selector ? t.selector : null,
						host: t.host || eO,
						propMetadata: o,
						inputs: t.inputs || eA,
						outputs: t.outputs || eA,
						queries: fU(e, o, fH),
						lifecycle: { usesOnChanges: r.hasLifecycleHook(e, "ngOnChanges") },
						typeSourceSpan: null,
						usesInheritance:
							Object.getPrototypeOf(e.prototype) !== Object.prototype,
						exportAs: (function (e) {
							return void 0 === e ? null : fq(e);
						})(t.exportAs),
						providers: t.providers || null,
						viewQueries: fU(e, o, fz),
						isStandalone: !!t.standalone,
						hostDirectives:
							(null === (n = t.hostDirectives) || void 0 === n
								? void 0
								: n.map((e) =>
										"function" == typeof e ? { directive: e } : e,
								  )) || null,
					};
				}
				function fB(e) {
					let t = Object.prototype,
						n = Object.getPrototypeOf(e.prototype).constructor;
					for (; n && n !== t; )
						!e8(n) &&
							!e6(n) &&
							(function (e) {
								let t = rW();
								if (fG.some((n) => t.hasLifecycleHook(e, n))) return !0;
								let n = t.propMetadata(e);
								for (let e in n) {
									let t = n[e];
									for (let e = 0; e < t.length; e++) {
										let n = t[e],
											r = n.ngMetadataName;
										if (
											fW(n) ||
											fH(n) ||
											fz(n) ||
											"Output" === r ||
											"HostBinding" === r ||
											"HostListener" === r
										)
											return !0;
									}
								}
								return !1;
							})(n) &&
							fN(n, null),
							(n = Object.getPrototypeOf(n));
				}
				function fU(e, t, n) {
					let r = [];
					for (let o in t)
						if (t.hasOwnProperty(o)) {
							let i = t[o];
							i.forEach((t) => {
								if (n(t)) {
									if (!t.selector)
										throw Error(
											`Can't construct a query for the property "${o}" of "${P(
												e,
											)}" since the query selector wasn't defined.`,
										);
									if (i.some(fW))
										throw Error(
											"Cannot combine @Input decorators with query decorators",
										);
									r.push(
										(function (e, t) {
											var n;
											return {
												propertyName: e,
												predicate:
													"string" == typeof (n = t.selector) ? fq(n) : M(n),
												descendants: t.descendants,
												first: t.first,
												read: t.read ? t.read : null,
												static: !!t.static,
												emitDistinctChangesOnly: !!t.emitDistinctChangesOnly,
											};
										})(o, t),
									);
								}
							});
						}
					return r;
				}
				function fH(e) {
					let t = e.ngMetadataName;
					return "ContentChild" === t || "ContentChildren" === t;
				}
				function fz(e) {
					let t = e.ngMetadataName;
					return "ViewChild" === t || "ViewChildren" === t;
				}
				function fW(e) {
					return "Input" === e.ngMetadataName;
				}
				function fq(e) {
					return e.split(",").map((e) => e.trim());
				}
				let fG = [
					"ngOnChanges",
					"ngOnInit",
					"ngOnDestroy",
					"ngDoCheck",
					"ngAfterViewInit",
					"ngAfterViewChecked",
					"ngAfterContentInit",
					"ngAfterContentChecked",
				];
				function fZ(e, t) {
					return {
						type: e,
						name: e.name,
						pipeName: t.name,
						pure: void 0 === t.pure || t.pure,
						isStandalone: !!t.standalone,
					};
				}
				let fY = rc(
					"Directive",
					(e = {}) => e,
					void 0,
					void 0,
					(e, t) => fN(e, t),
				);
				rc(
					"Component",
					(e = {}) => m._({ changeDetection: eS.Default }, e),
					fY,
					void 0,
					(e, t) => {
						var n, r, o, i;
						let s;
						return (
							(n = e),
							(r = t),
							("undefined" == typeof ngDevMode || ngDevMode) && ef(),
							(s = null),
							void ((o = n),
							rQ((i = r)) && (rZ.set(o, i), rY.add(o)),
							f$(n, r),
							Object.defineProperty(n, eI, {
								get: () => {
									if (null === s) {
										let e = r_({ usage: 0, kind: "component", type: n });
										if (rQ(r)) {
											let e = [`Component '${n.name}' is not resolved:`];
											throw (
												(r.templateUrl &&
													e.push(` - templateUrl: ${r.templateUrl}`),
												r.styleUrls &&
													r.styleUrls.length &&
													e.push(
														` - styleUrls: ${JSON.stringify(r.styleUrls)}`,
													),
												e.push(
													"Did you run and wait for 'resolveComponentResources()'?",
												),
												Error(e.join("\n")))
											);
										}
										let t = fD,
											o = r.preserveWhitespaces;
										void 0 === o &&
											(o =
												null !== t &&
												void 0 !== t.preserveWhitespaces &&
												t.preserveWhitespaces);
										let i = r.encapsulation;
										void 0 === i &&
											(i =
												null !== t && void 0 !== t.defaultEncapsulation
													? t.defaultEncapsulation
													: eE.Emulated);
										let l = r.templateUrl || `ng:///${n.name}/template.html`,
											a = g._(m._({}, fV(n, r)), {
												typeSourceSpan: e.createParseSourceSpan(
													"Component",
													n.name,
													l,
												),
												template: r.template || "",
												preserveWhitespaces: o,
												styles: r.styles || eA,
												animations: r.animations,
												declarations: [],
												changeDetection: r.changeDetection,
												encapsulation: i,
												interpolation: r.interpolation,
												viewProviders: r.viewProviders || null,
											});
										fR++;
										try {
											if (
												(a.usesInheritance && fB(n),
												(s = e.compileComponent(fx, l, a)),
												r.standalone)
											) {
												let e = rD(r.imports || eA),
													{ directiveDefs: t, pipeDefs: o } = (function (e, t) {
														let n = null,
															r = null;
														return {
															directiveDefs: () => {
																if (null === n) {
																	n = [e6(e)];
																	let r = new Set();
																	for (let o of t) {
																		ngDevMode &&
																			(function (e, t) {
																				if (C(e) && !(e = M(e)))
																					throw Error(
																						`Expected forwardRef function, imported from "${P(
																							t,
																						)}", to return a standalone entity or NgModule but got "${
																							P(e) || e
																						}".`,
																					);
																				if (null == te(e)) {
																					let r = e6(e) || e8(e) || e7(e);
																					if (null != r) {
																						if (!r.standalone) {
																							var n;
																							throw Error(
																								`The "${P(e)}" ${
																									e6((n = e))
																										? "component"
																										: e8(n)
																										? "directive"
																										: e7(n)
																										? "pipe"
																										: "type"
																								}, imported from "${P(
																									t,
																								)}", is not standalone. Did you forget to add the standalone: true flag?`,
																							);
																						}
																					} else {
																						if (fw(e))
																							throw Error(
																								`A module with providers was imported from "${P(
																									t,
																								)}". Modules with providers are not supported in standalone components imports.`,
																							);
																						throw Error(
																							`The "${P(
																								e,
																							)}" type, imported from "${P(
																								t,
																							)}", must be a standalone component / directive / pipe or an NgModule. Did you forget to add the required @Component / @Directive / @Pipe or @NgModule annotation?`,
																						);
																					}
																				}
																			})(o, e);
																		let t = M(o);
																		if (!r.has(t)) {
																			if ((r.add(t), te(t))) {
																				let e = fk(t);
																				for (let t of e.exported.directives) {
																					let e = e6(t) || e8(t);
																					e &&
																						!r.has(t) &&
																						(r.add(t), n.push(e));
																				}
																			} else {
																				let e = e6(t) || e8(t);
																				e && n.push(e);
																			}
																		}
																	}
																}
																return n;
															},
															pipeDefs: () => {
																if (null === r) {
																	r = [];
																	let e = new Set();
																	for (let n of t) {
																		let t = M(n);
																		if (!e.has(t)) {
																			if ((e.add(t), te(t))) {
																				let n = fk(t);
																				for (let t of n.exported.pipes) {
																					let n = e7(t);
																					n &&
																						!e.has(t) &&
																						(e.add(t), r.push(n));
																				}
																			} else {
																				let e = e7(t);
																				e && r.push(e);
																			}
																		}
																	}
																}
																return r;
															},
														};
													})(n, e);
												(s.directiveDefs = t),
													(s.pipeDefs = o),
													(s.dependencies = () => e.map(M));
											}
										} finally {
											fR--;
										}
										if (
											(0 === fR &&
												!(function () {
													if (!fS) {
														fS = !0;
														try {
															for (let e = fC.length - 1; e >= 0; e--) {
																let { moduleType: t, ngModule: n } = fC[e];
																n.declarations &&
																	n.declarations.every(fE) &&
																	(fC.splice(e, 1),
																	(function (e, t) {
																		let n = rD(t.declarations || eA),
																			r = fk(e);
																		n.forEach((t) => {
																			if ((t = M(t)).hasOwnProperty(eI)) {
																				let e = t,
																					n = e6(e);
																				fT(n, r);
																			} else
																				!t.hasOwnProperty(eP) &&
																					!t.hasOwnProperty(eT) &&
																					(t.ngSelectorScope = e);
																		});
																	})(t, n));
															}
														} finally {
															fS = !1;
														}
													}
												})(),
											(function (e) {
												return void 0 !== e.ngSelectorScope;
											})(n))
										) {
											let e = fk(n.ngSelectorScope);
											fT(s, e);
										}
										if (r.schemas) {
											if (r.standalone) s.schemas = r.schemas;
											else
												throw Error(
													`The 'schemas' was specified for the ${P(
														n,
													)} but is only valid on a component that is standalone.`,
												);
										} else r.standalone && (s.schemas = []);
									}
									return s;
								},
								configurable: !!ngDevMode,
							}))
						);
					},
				),
					rc(
						"Pipe",
						(e) => m._({ pure: !0 }, e),
						void 0,
						void 0,
						(e, t) => {
							var n, r;
							let o, i;
							return (
								(n = e),
								(r = t),
								(o = null),
								(i = null),
								void (Object.defineProperty(n, eF, {
									get: () => {
										if (null === i) {
											let e = fZ(n, r),
												t = r_({ usage: 0, kind: "pipe", type: e.type });
											i = t.compileFactory(fx, `ng:///${e.name}/ɵfac.js`, {
												name: e.name,
												type: e.type,
												typeArgumentCount: 0,
												deps: rq(n),
												target: t.FactoryTarget.Pipe,
											});
										}
										return i;
									},
									configurable: !!ngDevMode,
								}),
								Object.defineProperty(n, eT, {
									get: () => {
										if (null === o) {
											let e = fZ(n, r),
												t = r_({ usage: 0, kind: "pipe", type: e.type });
											o = t.compilePipe(fx, `ng:///${e.name}/ɵpipe.js`, e);
										}
										return o;
									},
									configurable: !!ngDevMode,
								}))
							);
						},
					),
					rp("Input", (e) =>
						e ? ("string" == typeof e ? { alias: e } : e) : {},
					),
					rp("Output", (e) => ({ alias: e })),
					rp("HostBinding", (e) => ({ hostPropertyName: e })),
					rp("HostListener", (e, t) => ({ eventName: e, args: t }));
				let fQ = rc(
						"NgModule",
						(e) => e,
						void 0,
						void 0,
						(e, t) =>
							(function (e, t = {}) {
								var n, r;
								(function (e, t, n = !1) {
									ngDevMode && Z(e, "Required value moduleType"),
										ngDevMode && Z(t, "Required value ngModule");
									let r = rD(t.declarations || eA),
										o = null;
									Object.defineProperty(e, ek, {
										configurable: !0,
										get: () => {
											if (null === o) {
												if (ngDevMode && t.imports && t.imports.indexOf(e) > -1)
													throw Error(`'${P(e)}' module can't import itself`);
												let n = r_({ usage: 0, kind: "NgModule", type: e });
												!(o = n.compileNgModule(fx, `ng:///${e.name}/ɵmod.js`, {
													type: e,
													bootstrap: rD(t.bootstrap || eA).map(M),
													declarations: r.map(M),
													imports: rD(t.imports || eA)
														.map(M)
														.map(fF),
													exports: rD(t.exports || eA)
														.map(M)
														.map(fF),
													schemas: t.schemas ? rD(t.schemas) : null,
													id: t.id || null,
												})).schemas && (o.schemas = []);
											}
											return o;
										},
									});
									let i = null;
									Object.defineProperty(e, eF, {
										get: () => {
											if (null === i) {
												let t = r_({ usage: 0, kind: "NgModule", type: e });
												i = t.compileFactory(fx, `ng:///${e.name}/ɵfac.js`, {
													name: e.name,
													type: e,
													deps: rq(e),
													target: t.FactoryTarget.NgModule,
													typeArgumentCount: 0,
												});
											}
											return i;
										},
										configurable: !!ngDevMode,
									});
									let s = null;
									Object.defineProperty(e, ei, {
										get: () => {
											if (null === s) {
												ngDevMode && fO(e, n);
												let r = {
														name: e.name,
														type: e,
														providers: t.providers || eA,
														imports: [
															(t.imports || eA).map(M),
															(t.exports || eA).map(M),
														],
													},
													o = r_({ usage: 0, kind: "NgModule", type: e });
												s = o.compileInjector(fx, `ng:///${e.name}/ɵinj.js`, r);
											}
											return s;
										},
										configurable: !!ngDevMode,
									});
								})(e, t),
									void 0 !== t.id && rX(e, t.id),
									(n = e),
									(r = t),
									fC.push({ moduleType: n, ngModule: r });
							})(e, t),
					),
					fK = new iq("Application Initializer"),
					fJ = (() => {
						class e {
							runInitializers() {
								if (this.initialized) return;
								let e = [];
								for (let t of this.appInits) {
									let n = t();
									if (aT(n)) e.push(n);
									else if (ak(n)) {
										let t = new Promise((e, t) => {
											n.subscribe({ complete: e, error: t });
										});
										e.push(t);
									}
								}
								let t = () => {
									(this.done = !0), this.resolve();
								};
								Promise.all(e)
									.then(() => {
										t();
									})
									.catch((e) => {
										this.reject(e);
									}),
									0 === e.length && t(),
									(this.initialized = !0);
							}
							constructor() {
								var e;
								if (
									((this.initialized = !1),
									(this.done = !1),
									(this.donePromise = new Promise((e, t) => {
										(this.resolve = e), (this.reject = t);
									})),
									(this.appInits =
										null !== (e = ex(fK, { optional: !0 })) && void 0 !== e
											? e
											: []),
									("undefined" == typeof ngDevMode || ngDevMode) &&
										!Array.isArray(this.appInits))
								)
									throw new O(
										-209,
										`Unexpected type of the \`APP_INITIALIZER\` token value (expected an array, but got ${typeof this
											.appInits}). Please check that the \`APP_INITIALIZER\` token is configured as a \`multi: true\` provider.`,
									);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = J({ token: e, factory: e.ɵfac, providedIn: "root" })),
							e
						);
					})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						fJ,
						[{ type: s1, args: [{ providedIn: "root" }] }],
						function () {
							return [];
						},
						null,
					);
				let fX = (() => {
					class e {
						log(e) {
							console.log(e);
						}
						warn(e) {
							console.warn(e);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({
							token: e,
							factory: e.ɵfac,
							providedIn: "platform",
						})),
						e
					);
				})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						fX,
						[{ type: s1, args: [{ providedIn: "platform" }] }],
						null,
						null,
					);
				let f0 = new iq("LocaleId", {
						providedIn: "root",
						factory: () =>
							ex(f0, ea.Optional | ea.SkipSelf) ||
							("undefined" != typeof ngI18nClosureMode &&
							ngI18nClosureMode &&
							"undefined" != typeof goog &&
							"en" !== goog.LOCALE
								? goog.LOCALE
								: ("undefined" != typeof $localize && $localize.locale) || du),
					}),
					f1 = new iq("DefaultCurrencyCode", {
						providedIn: "root",
						factory: () => "USD",
					});
				new iq("Translations"), new iq("TranslationsFormat");
				var f5 =
					(((f5 = f5 || {})[(f5.Error = 0)] = "Error"),
					(f5[(f5.Warning = 1)] = "Warning"),
					(f5[(f5.Ignore = 2)] = "Ignore"),
					f5);
				class f2 {
					constructor(e, t) {
						(this.ngModuleFactory = e), (this.componentFactories = t);
					}
				}
				let f3 = (() => {
					class e {
						compileModuleSync(e) {
							return new cf(e);
						}
						compileModuleAsync(e) {
							return Promise.resolve(this.compileModuleSync(e));
						}
						compileModuleAndAllComponentsSync(e) {
							let t = this.compileModuleSync(e),
								n = te(e),
								r = sB(n.declarations).reduce((e, t) => {
									let n = e6(t);
									return n && e.push(new lz(n)), e;
								}, []);
							return new f2(t, r);
						}
						compileModuleAndAllComponentsAsync(e) {
							return Promise.resolve(this.compileModuleAndAllComponentsSync(e));
						}
						clearCache() {}
						clearCacheFor(e) {}
						getModuleId(e) {}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({ token: e, factory: e.ɵfac, providedIn: "root" })),
						e
					);
				})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(f3, [{ type: s1, args: [{ providedIn: "root" }] }], null, null);
				let f4 = new iq("compilerOptions");
				function f6(e) {
					ngDevMode && Z(e, "component"),
						sP(ou(e)),
						c_(e).forEach((e) =>
							(function (e) {
								let t = ou(e);
								lR(t[1], t, e);
							})(e),
						);
				}
				let f8 = !1;
				function f7(e, t) {
					if (
						("undefined" == typeof COMPILED || !COMPILED) &&
						(ngDevMode && Z(t, "function not defined"), ec)
					) {
						let n = ec.ng;
						!n && (n = ec.ng = {}), (n[e] = t);
					}
				}
				let f9 = Promise.resolve(0);
				function he(e) {
					"undefined" == typeof Zone
						? f9.then(() => {
								e && e.apply(null, null);
						  })
						: Zone.current.scheduleMicroTask("scheduleMicrotask", e);
				}
				function ht(...e) {}
				class hn {
					onScheduleTask(e, t, n, r) {
						return (
							(r.consoleTask = this.createTask(`Zone - ${r.source || r.type}`)),
							e.scheduleTask(n, r)
						);
					}
					onInvokeTask(e, t, n, r, o, i) {
						let s;
						return (s = r.consoleTask
							? r.consoleTask.run(() => e.invokeTask(n, r, o, i))
							: e.invokeTask(n, r, o, i));
					}
					constructor(e, t = console) {
						var n;
						(this.name = "asyncStackTagging for " + e),
							(this.createTask =
								null !== (n = null == t ? void 0 : t.createTask) && void 0 !== n
									? n
									: () => null);
					}
				}
				class hr {
					static isInAngularZone() {
						return (
							"undefined" != typeof Zone &&
							!0 === Zone.current.get("isAngularZone")
						);
					}
					static assertInAngularZone() {
						if (!hr.isInAngularZone())
							throw new O(
								909,
								ngDevMode && "Expected to be in Angular Zone, but it is not!",
							);
					}
					static assertNotInAngularZone() {
						if (hr.isInAngularZone())
							throw new O(
								909,
								ngDevMode && "Expected to not be in Angular Zone, but it is!",
							);
					}
					run(e, t, n) {
						return this._inner.run(e, t, n);
					}
					runTask(e, t, n, r) {
						let o = this._inner,
							i = o.scheduleEventTask("NgZoneEvent: " + r, e, ho, ht, ht);
						try {
							return o.runTask(i, t, n);
						} finally {
							o.cancelTask(i);
						}
					}
					runGuarded(e, t, n) {
						return this._inner.runGuarded(e, t, n);
					}
					runOutsideAngular(e) {
						return this._outer.run(e);
					}
					constructor({
						enableLongStackTrace: e = !1,
						shouldCoalesceEventChangeDetection: t = !1,
						shouldCoalesceRunChangeDetection: n = !1,
					}) {
						if (
							((this.hasPendingMacrotasks = !1),
							(this.hasPendingMicrotasks = !1),
							(this.isStable = !0),
							(this.onUnstable = new c0(!1)),
							(this.onMicrotaskEmpty = new c0(!1)),
							(this.onStable = new c0(!1)),
							(this.onError = new c0(!1)),
							"undefined" == typeof Zone)
						)
							throw new O(
								908,
								ngDevMode && "In this configuration Angular requires Zone.js",
							);
						Zone.assertZonePatched();
						(this._nesting = 0),
							(this._outer = this._inner = Zone.current),
							ngDevMode && (this._inner = this._inner.fork(new hn("Angular"))),
							Zone.TaskTrackingZoneSpec &&
								(this._inner = this._inner.fork(
									new Zone.TaskTrackingZoneSpec(),
								)),
							e &&
								Zone.longStackTraceZoneSpec &&
								(this._inner = this._inner.fork(Zone.longStackTraceZoneSpec)),
							(this.shouldCoalesceEventChangeDetection = !n && t),
							(this.shouldCoalesceRunChangeDetection = n),
							(this.lastRequestAnimationFrameId = -1),
							(this.nativeRequestAnimationFrame = (function () {
								let e = ec.requestAnimationFrame,
									t = ec.cancelAnimationFrame;
								if ("undefined" != typeof Zone && e && t) {
									let n = e[Zone.__symbol__("OriginalDelegate")];
									n && (e = n);
									let r = t[Zone.__symbol__("OriginalDelegate")];
									r && (t = r);
								}
								return {
									nativeRequestAnimationFrame: e,
									nativeCancelAnimationFrame: t,
								};
							})().nativeRequestAnimationFrame),
							(function (e) {
								let t = () => {
									var t;
									(t = e).isCheckStableRunning ||
										-1 !== t.lastRequestAnimationFrameId ||
										((t.lastRequestAnimationFrameId =
											t.nativeRequestAnimationFrame.call(ec, () => {
												!t.fakeTopEventTask &&
													(t.fakeTopEventTask = Zone.root.scheduleEventTask(
														"fakeTopEventTask",
														() => {
															(t.lastRequestAnimationFrameId = -1),
																hs(t),
																(t.isCheckStableRunning = !0),
																hi(t),
																(t.isCheckStableRunning = !1);
														},
														void 0,
														() => {},
														() => {},
													)),
													t.fakeTopEventTask.invoke();
											})),
										hs(t));
								};
								e._inner = e._inner.fork({
									name: "angular",
									properties: { isAngularZone: !0 },
									onInvokeTask: (n, r, o, i, s, l) => {
										try {
											return hl(e), n.invokeTask(o, i, s, l);
										} finally {
											((e.shouldCoalesceEventChangeDetection &&
												"eventTask" === i.type) ||
												e.shouldCoalesceRunChangeDetection) &&
												t(),
												ha(e);
										}
									},
									onInvoke: (n, r, o, i, s, l, a) => {
										try {
											return hl(e), n.invoke(o, i, s, l, a);
										} finally {
											e.shouldCoalesceRunChangeDetection && t(), ha(e);
										}
									},
									onHasTask: (t, n, r, o) => {
										t.hasTask(r, o),
											n === r &&
												("microTask" == o.change
													? ((e._hasPendingMicrotasks = o.microTask),
													  hs(e),
													  hi(e))
													: "macroTask" == o.change &&
													  (e.hasPendingMacrotasks = o.macroTask));
									},
									onHandleError: (t, n, r, o) => (
										t.handleError(r, o),
										e.runOutsideAngular(() => e.onError.emit(o)),
										!1
									),
								});
							})(this);
					}
				}
				let ho = {};
				function hi(e) {
					if (0 == e._nesting && !e.hasPendingMicrotasks && !e.isStable)
						try {
							e._nesting++, e.onMicrotaskEmpty.emit(null);
						} finally {
							if ((e._nesting--, !e.hasPendingMicrotasks))
								try {
									e.runOutsideAngular(() => e.onStable.emit(null));
								} finally {
									e.isStable = !0;
								}
						}
				}
				function hs(e) {
					e._hasPendingMicrotasks ||
					((e.shouldCoalesceEventChangeDetection ||
						e.shouldCoalesceRunChangeDetection) &&
						-1 !== e.lastRequestAnimationFrameId)
						? (e.hasPendingMicrotasks = !0)
						: (e.hasPendingMicrotasks = !1);
				}
				function hl(e) {
					e._nesting++,
						e.isStable && ((e.isStable = !1), e.onUnstable.emit(null));
				}
				function ha(e) {
					e._nesting--, hi(e);
				}
				class hu {
					run(e, t, n) {
						return e.apply(t, n);
					}
					runGuarded(e, t, n) {
						return e.apply(t, n);
					}
					runOutsideAngular(e) {
						return e();
					}
					runTask(e, t, n, r) {
						return e.apply(t, n);
					}
					constructor() {
						(this.hasPendingMicrotasks = !1),
							(this.hasPendingMacrotasks = !1),
							(this.isStable = !0),
							(this.onUnstable = new c0()),
							(this.onMicrotaskEmpty = new c0()),
							(this.onStable = new c0()),
							(this.onError = new c0());
					}
				}
				let hd = new iq(ngDevMode ? "isStable Observable" : "", {
					providedIn: "root",
					factory: hc,
				});
				function hc() {
					let e = ex(hr),
						t = !0,
						n = new v.Observable((n) => {
							(t =
								e.isStable &&
								!e.hasPendingMacrotasks &&
								!e.hasPendingMicrotasks),
								e.runOutsideAngular(() => {
									n.next(t), n.complete();
								});
						}),
						r = new v.Observable((n) => {
							let r;
							e.runOutsideAngular(() => {
								r = e.onStable.subscribe(() => {
									hr.assertNotInAngularZone(),
										he(() => {
											!t &&
												!e.hasPendingMacrotasks &&
												!e.hasPendingMicrotasks &&
												((t = !0), n.next(!0));
										});
								});
							});
							let o = e.onUnstable.subscribe(() => {
								hr.assertInAngularZone(),
									t &&
										((t = !1),
										e.runOutsideAngular(() => {
											n.next(!1);
										}));
							});
							return () => {
								r.unsubscribe(), o.unsubscribe();
							};
						});
					return (0, v.merge)(n, r.pipe((0, y.share)()));
				}
				let hf = new iq(""),
					hh = new iq(""),
					hp = (() => {
						class e {
							_watchAngularEvents() {
								this._ngZone.onUnstable.subscribe({
									next: () => {
										(this._didWork = !0), (this._isZoneStable = !1);
									},
								}),
									this._ngZone.runOutsideAngular(() => {
										this._ngZone.onStable.subscribe({
											next: () => {
												hr.assertNotInAngularZone(),
													he(() => {
														(this._isZoneStable = !0),
															this._runCallbacksIfReady();
													});
											},
										});
									});
							}
							increasePendingRequestCount() {
								return (
									(this._pendingCount += 1),
									(this._didWork = !0),
									this._pendingCount
								);
							}
							decreasePendingRequestCount() {
								if (((this._pendingCount -= 1), this._pendingCount < 0))
									throw Error("pending async requests below zero");
								return this._runCallbacksIfReady(), this._pendingCount;
							}
							isStable() {
								return (
									this._isZoneStable &&
									0 === this._pendingCount &&
									!this._ngZone.hasPendingMacrotasks
								);
							}
							_runCallbacksIfReady() {
								if (this.isStable())
									he(() => {
										for (; 0 !== this._callbacks.length; ) {
											let e = this._callbacks.pop();
											clearTimeout(e.timeoutId), e.doneCb(this._didWork);
										}
										this._didWork = !1;
									});
								else {
									let e = this.getPendingTasks();
									(this._callbacks = this._callbacks.filter(
										(t) =>
											!(t.updateCb && t.updateCb(e)) ||
											(clearTimeout(t.timeoutId), !1),
									)),
										(this._didWork = !0);
								}
							}
							getPendingTasks() {
								return this.taskTrackingZone
									? this.taskTrackingZone.macroTasks.map((e) => ({
											source: e.source,
											creationLocation: e.creationLocation,
											data: e.data,
									  }))
									: [];
							}
							addCallback(e, t, n) {
								let r = -1;
								t &&
									t > 0 &&
									(r = setTimeout(() => {
										(this._callbacks = this._callbacks.filter(
											(e) => e.timeoutId !== r,
										)),
											e(this._didWork, this.getPendingTasks());
									}, t)),
									this._callbacks.push({
										doneCb: e,
										timeoutId: r,
										updateCb: n,
									});
							}
							whenStable(e, t, n) {
								if (n && !this.taskTrackingZone)
									throw Error(
										'Task tracking zone is required when passing an update callback to whenStable(). Is "zone.js/plugins/task-tracking" loaded?',
									);
								this.addCallback(e, t, n), this._runCallbacksIfReady();
							}
							getPendingRequestCount() {
								return this._pendingCount;
							}
							registerApplication(e) {
								this.registry.registerApplication(e, this);
							}
							unregisterApplication(e) {
								this.registry.unregisterApplication(e);
							}
							findProviders(e, t, n) {
								return [];
							}
							constructor(e, t, n) {
								(this._ngZone = e),
									(this.registry = t),
									(this._pendingCount = 0),
									(this._isZoneStable = !0),
									(this._didWork = !1),
									(this._callbacks = []),
									(this.taskTrackingZone = null),
									!u &&
										((function (e) {
											u = e;
										})(n),
										n.addToWindow(t)),
									this._watchAngularEvents(),
									e.run(() => {
										this.taskTrackingZone =
											"undefined" == typeof Zone
												? null
												: Zone.current.get("TaskTrackingZone");
									});
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(e_(hr), e_(hm), e_(hh));
							}),
							(e.ɵprov = J({ token: e, factory: e.ɵfac })),
							e
						);
					})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						hp,
						[{ type: s1 }],
						function () {
							return [
								{ type: hr },
								{ type: hm },
								{ type: void 0, decorators: [{ type: r$, args: [hh] }] },
							];
						},
						null,
					);
				let hm = (() => {
					class e {
						registerApplication(e, t) {
							this._applications.set(e, t);
						}
						unregisterApplication(e) {
							this._applications.delete(e);
						}
						unregisterAllApplications() {
							this._applications.clear();
						}
						getTestability(e) {
							return this._applications.get(e) || null;
						}
						getAllTestabilities() {
							return Array.from(this._applications.values());
						}
						getAllRootElements() {
							return Array.from(this._applications.keys());
						}
						findTestabilityInTree(e, t = !0) {
							var n;
							return null !==
								(n =
									null == u ? void 0 : u.findTestabilityInTree(this, e, t)) &&
								void 0 !== n
								? n
								: null;
						}
						constructor() {
							this._applications = new Map();
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({
							token: e,
							factory: e.ɵfac,
							providedIn: "platform",
						})),
						e
					);
				})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						hm,
						[{ type: s1, args: [{ providedIn: "platform" }] }],
						null,
						null,
					);
				let hg = null,
					hv = new iq("AllowMultipleToken"),
					hy = new iq("PlatformDestroyListeners"),
					hb = new iq("appBootstrapListener");
				class h_ {
					constructor(e, t) {
						(this.name = e), (this.token = t);
					}
				}
				function hj(e, t, n = []) {
					let r = `Platform: ${t}`,
						o = new iq(r);
					return (t = []) => {
						let i = hx();
						if (!i || i.injector.get(hv, !1)) {
							let i = [...n, ...t, { provide: o, useValue: !0 }];
							e
								? e(i)
								: !(function (e) {
										var t;
										if (hg && !hg.get(hv, !1))
											throw new O(
												400,
												ngDevMode &&
													"There can be only one platform. Destroy the previous one to create a new one.",
											);
										ngDevMode &&
											(f8 ||
												((f8 = !0),
												f7("ɵsetProfiler", tU),
												f7("getDirectiveMetadata", cD),
												f7("getComponent", cv),
												f7("getContext", cy),
												f7("getListeners", cM),
												f7("getOwningComponent", cb),
												f7("getHostElement", cw),
												f7("getInjector", cj),
												f7("getRootComponents", c_),
												f7("getDirectives", cx),
												f7("applyChanges", f6))),
											(t = () => {
												throw new O(
													600,
													ngDevMode &&
														"Writing to signals is not allowed in a `computed` or an `effect` by default. Use `allowSignalWrites` in the `CreateEffectOptions` to enable this inside effects.",
												);
											}),
											(hg = e);
										let n = e.get(hD);
										return (
											(function (e) {
												let t = e.get(sa, null);
												null == t || t.forEach((e) => e());
											})(e),
											n
										);
								  })(
										(function (e = [], t) {
											return s3.create({
												name: t,
												providers: [
													{ provide: i8, useValue: "platform" },
													{
														provide: hy,
														useValue: new Set([() => (hg = null)]),
													},
													...e,
												],
											});
										})(i, r),
								  );
						}
						return (function (e) {
							let t = hx();
							if (!t) throw new O(401, ngDevMode && "No platform exists!");
							if (
								("undefined" == typeof ngDevMode || ngDevMode) &&
								!t.injector.get(e, null)
							)
								throw new O(
									400,
									"A platform with a different configuration has been created. Please destroy it first.",
								);
							return t;
						})(o);
					};
				}
				function hx() {
					var e;
					return null !== (e = null == hg ? void 0 : hg.get(hD)) && void 0 !== e
						? e
						: null;
				}
				let hD = (() => {
					class e {
						bootstrapModuleFactory(e, t) {
							let n = (function (e = "zone.js", t) {
								return "noop" === e
									? new hu()
									: "zone.js" === e
									? new hr(t)
									: e;
							})(
								null == t ? void 0 : t.ngZone,
								(function (e) {
									var t, n;
									return {
										enableLongStackTrace:
											"undefined" != typeof ngDevMode && !!ngDevMode,
										shouldCoalesceEventChangeDetection:
											null !== (t = null == e ? void 0 : e.eventCoalescing) &&
											void 0 !== t &&
											t,
										shouldCoalesceRunChangeDetection:
											null !== (n = null == e ? void 0 : e.runCoalescing) &&
											void 0 !== n &&
											n,
									};
								})({
									eventCoalescing: null == t ? void 0 : t.ngZoneEventCoalescing,
									runCoalescing: null == t ? void 0 : t.ngZoneRunCoalescing,
								}),
							);
							return n.run(() => {
								var t, r, o;
								let i =
									((t = e.moduleType),
									(r = this.injector),
									(o = (function (e) {
										return [
											{ provide: hr, useFactory: e },
											{
												provide: iG,
												multi: !0,
												useFactory: () => {
													let e = ex(hO, { optional: !0 });
													if (
														("undefined" == typeof ngDevMode || ngDevMode) &&
														null === e
													)
														throw new O(
															402,
															"A required Injectable was not found in the dependency injection tree. If you are bootstrapping an NgModule, make sure that the `BrowserModule` is imported.",
														);
													return () => e.initialize();
												},
											},
											{ provide: hS, useFactory: hE },
											{ provide: hd, useFactory: hc },
										];
									})(() => n)),
									new cc(t, r, o));
								if (
									("undefined" == typeof ngDevMode || ngDevMode) &&
									null !== i.injector.get(hA, null)
								)
									throw new O(
										207,
										"`bootstrapModule` does not support `provideZoneChangeDetection`. Use `BootstrapOptions` instead.",
									);
								let s = i.injector.get(sk, null);
								if (
									("undefined" == typeof ngDevMode || ngDevMode) &&
									null === s
								)
									throw new O(
										402,
										"No ErrorHandler. Is platform module (BrowserModule) included?",
									);
								return (
									n.runOutsideAngular(() => {
										let e = n.onError.subscribe({
											next: (e) => {
												s.handleError(e);
											},
										});
										i.onDestroy(() => {
											hM(this._modules, i), e.unsubscribe();
										});
									}),
									(function (e, t, n) {
										try {
											let r = n();
											if (aT(r))
												return r.catch((n) => {
													throw (
														(t.runOutsideAngular(() => e.handleError(n)), n)
													);
												});
											return r;
										} catch (n) {
											throw (t.runOutsideAngular(() => e.handleError(n)), n);
										}
									})(s, n, () => {
										let e = i.injector.get(fJ);
										return (
											e.runInitializers(),
											e.donePromise.then(() => {
												var e;
												let t = i.injector.get(f0, du);
												return (
													Z((e = t || du), "Expected localeId to be defined"),
													"string" == typeof e &&
														(dh = e.toLowerCase().replace(/_/g, "-")),
													this._moduleDoBootstrap(i),
													i
												);
											})
										);
									})
								);
							});
						}
						bootstrapModule(e, t = []) {
							let n = (function e(t, n) {
								return Array.isArray(n) ? n.reduce(e, t) : m._({}, t, n);
							})({}, t);
							return (function (e, t, n) {
								var r;
								ngDevMode &&
									(function (
										e,
										t = "Type passed in is not NgModuleType, it does not have 'ɵmod' property.",
									) {
										!te(e) && Y(t);
									})(n);
								let o = new cf(n);
								if ("undefined" != typeof ngJitMode && !ngJitMode)
									return Promise.resolve(o);
								let i = e.get(f4, []).concat(t);
								if (
									(!(function (e) {
										if (null !== fD) {
											if (e.defaultEncapsulation !== fD.defaultEncapsulation) {
												ngDevMode &&
													console.error(
														"Provided value for `defaultEncapsulation` can not be changed once it has been set.",
													);
												return;
											}
											if (e.preserveWhitespaces !== fD.preserveWhitespaces) {
												ngDevMode &&
													console.error(
														"Provided value for `preserveWhitespaces` can not be changed once it has been set.",
													);
												return;
											}
										}
										fD = e;
									})({
										defaultEncapsulation: hC(
											i.map((e) => e.defaultEncapsulation),
										),
										preserveWhitespaces: hC(
											i.map((e) => e.preserveWhitespaces),
										),
									}),
									0 === rZ.size)
								)
									return Promise.resolve(o);
								let s = i.flatMap((e) =>
									null !== (r = e.providers) && void 0 !== r ? r : [],
								);
								if (0 === s.length) return Promise.resolve(o);
								let l = r_({ usage: 0, kind: "NgModule", type: n }),
									a = s3.create({ providers: s }),
									u = a.get(l.ResourceLoader);
								return (function (e) {
									let t = [],
										n = new Map();
									function r(t) {
										let r = n.get(t);
										if (!r) {
											let o = e(t);
											n.set(t, (r = o.then(rK)));
										}
										return r;
									}
									return (
										rZ.forEach((e, n) => {
											let o = [];
											e.templateUrl &&
												o.push(
													r(e.templateUrl).then((t) => {
														e.template = t;
													}),
												);
											let i = e.styleUrls,
												s = e.styles || (e.styles = []),
												l = e.styles.length;
											i &&
												i.forEach((t, n) => {
													s.push(""),
														o.push(
															r(t).then((r) => {
																(s[l + n] = r),
																	i.splice(i.indexOf(t), 1),
																	0 == i.length && (e.styleUrls = void 0);
															}),
														);
												});
											let a = Promise.all(o).then(() =>
												(function (e) {
													rY.delete(e);
												})(n),
											);
											t.push(a);
										}),
										(function () {
											let e = rZ;
											rZ = new Map();
										})(),
										Promise.all(t).then(() => void 0)
									);
								})((e) => Promise.resolve(u.get(e))).then(() => o);
							})(this.injector, n, e).then((e) =>
								this.bootstrapModuleFactory(e, n),
							);
						}
						_moduleDoBootstrap(e) {
							let t = e.injector.get(hw);
							if (e._bootstrapComponents.length > 0)
								e._bootstrapComponents.forEach((e) => t.bootstrap(e));
							else if (e.instance.ngDoBootstrap) e.instance.ngDoBootstrap(t);
							else
								throw new O(
									-403,
									ngDevMode &&
										`The module ${j(
											e.instance.constructor,
										)} was bootstrapped, but it does not declare "@NgModule.bootstrap" components nor a "ngDoBootstrap" method. Please define one of these.`,
								);
							this._modules.push(e);
						}
						onDestroy(e) {
							this._destroyListeners.push(e);
						}
						get injector() {
							return this._injector;
						}
						destroy() {
							if (this._destroyed)
								throw new O(
									404,
									ngDevMode && "The platform has already been destroyed!",
								);
							this._modules.slice().forEach((e) => e.destroy()),
								this._destroyListeners.forEach((e) => e());
							let e = this._injector.get(hy, null);
							e && (e.forEach((e) => e()), e.clear()), (this._destroyed = !0);
						}
						get destroyed() {
							return this._destroyed;
						}
						constructor(e) {
							(this._injector = e),
								(this._modules = []),
								(this._destroyListeners = []),
								(this._destroyed = !1);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(e_(s3));
						}),
						(e.ɵprov = J({
							token: e,
							factory: e.ɵfac,
							providedIn: "platform",
						})),
						e
					);
				})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						hD,
						[{ type: s1, args: [{ providedIn: "platform" }] }],
						function () {
							return [{ type: s3 }];
						},
						null,
					);
				let hw = (() => {
					class e {
						get destroyed() {
							return this._destroyed;
						}
						get injector() {
							return this._injector;
						}
						bootstrap(e, t) {
							let n;
							("undefined" == typeof ngDevMode || ngDevMode) &&
								this.warnIfDestroyed();
							let r = e instanceof sy,
								o = this._injector.get(fJ);
							if (!o.done) {
								let t = !r && e9(e);
								throw new O(
									405,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										"Cannot bootstrap as there are still asynchronous initializers running." +
											(t
												? ""
												: " Bootstrap components in the `ngDoBootstrap` method of the root module."),
								);
							}
							if (r) n = e;
							else {
								let t = this._injector.get(sj);
								n = t.resolveComponentFactory(e);
							}
							this.componentTypes.push(n.componentType);
							let i = n.isBoundToModule ? void 0 : this._injector.get(ca),
								s = t || n.selector,
								l = n.create(s3.NULL, [], s, i),
								a = l.location.nativeElement,
								u = l.injector.get(hf, null);
							if (
								(null == u || u.registerApplication(a),
								l.onDestroy(() => {
									this.detachView(l.hostView),
										hM(this.components, l),
										null == u || u.unregisterApplication(a);
								}),
								this._loadComponent(l),
								"undefined" == typeof ngDevMode || ngDevMode)
							) {
								let e = this._injector.get(fX);
								e.log("Angular is running in development mode.");
							}
							return l;
						}
						tick() {
							if (
								(("undefined" == typeof ngDevMode || ngDevMode) &&
									this.warnIfDestroyed(),
								this._runningTick)
							)
								throw new O(
									101,
									ngDevMode && "ApplicationRef.tick is called recursively",
								);
							try {
								for (let e of ((this._runningTick = !0), this._views))
									e.detectChanges();
								if ("undefined" == typeof ngDevMode || ngDevMode)
									for (let e of this._views) e.checkNoChanges();
							} catch (e) {
								this.internalErrorHandler(e);
							} finally {
								this._runningTick = !1;
							}
						}
						attachView(e) {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								this.warnIfDestroyed();
							this._views.push(e), e.attachToAppRef(this);
						}
						detachView(e) {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								this.warnIfDestroyed();
							hM(this._views, e), e.detachFromAppRef();
						}
						_loadComponent(e) {
							this.attachView(e.hostView), this.tick(), this.components.push(e);
							let t = this._injector.get(hb, []);
							if (ngDevMode && !Array.isArray(t))
								throw new O(
									-209,
									`Unexpected type of the \`APP_BOOTSTRAP_LISTENER\` token value (expected an array, but got ${typeof t}). Please check that the \`APP_BOOTSTRAP_LISTENER\` token is configured as a \`multi: true\` provider.`,
								);
							t.push(...this._bootstrapListeners), t.forEach((t) => t(e));
						}
						ngOnDestroy() {
							if (!this._destroyed)
								try {
									this._destroyListeners.forEach((e) => e()),
										this._views.slice().forEach((e) => e.destroy());
								} finally {
									(this._destroyed = !0),
										(this._views = []),
										(this._bootstrapListeners = []),
										(this._destroyListeners = []);
								}
						}
						onDestroy(e) {
							return (
								("undefined" == typeof ngDevMode || ngDevMode) &&
									this.warnIfDestroyed(),
								this._destroyListeners.push(e),
								() => hM(this._destroyListeners, e)
							);
						}
						destroy() {
							if (this._destroyed)
								throw new O(
									406,
									ngDevMode &&
										"This instance of the `ApplicationRef` has already been destroyed.",
								);
							let e = this._injector;
							e.destroy && !e.destroyed && e.destroy();
						}
						get viewCount() {
							return this._views.length;
						}
						warnIfDestroyed() {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								this._destroyed &&
								console.warn(
									A(
										406,
										"This instance of the `ApplicationRef` has already been destroyed.",
									),
								);
						}
						constructor() {
							(this._bootstrapListeners = []),
								(this._runningTick = !1),
								(this._destroyed = !1),
								(this._destroyListeners = []),
								(this._views = []),
								(this.internalErrorHandler = ex(hS)),
								(this.componentTypes = []),
								(this.components = []),
								(this.isStable = ex(hd)),
								(this._injector = ex(st));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({ token: e, factory: e.ɵfac, providedIn: "root" })),
						e
					);
				})();
				function hM(e, t) {
					let n = e.indexOf(t);
					n > -1 && e.splice(n, 1);
				}
				function hC(e) {
					for (let t = e.length - 1; t >= 0; t--)
						if (void 0 !== e[t]) return e[t];
				}
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(hw, [{ type: s1, args: [{ providedIn: "root" }] }], null, null);
				let hS = new iq(
					"undefined" == typeof ngDevMode || ngDevMode
						? "internal error handler"
						: "",
					{
						providedIn: "root",
						factory: () => {
							let e = ex(sk);
							return e.handleError.bind(void 0);
						},
					},
				);
				function hE() {
					let e = ex(hr),
						t = ex(sk);
					return (n) => e.runOutsideAngular(() => t.handleError(n));
				}
				let hO = (() => {
					class e {
						initialize() {
							!this._onMicrotaskEmptySubscription &&
								(this._onMicrotaskEmptySubscription =
									this.zone.onMicrotaskEmpty.subscribe({
										next: () => {
											this.zone.run(() => {
												this.applicationRef.tick();
											});
										},
									}));
						}
						ngOnDestroy() {
							var e;
							null === (e = this._onMicrotaskEmptySubscription) ||
								void 0 === e ||
								e.unsubscribe();
						}
						constructor() {
							(this.zone = ex(hr)), (this.applicationRef = ex(hw));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({ token: e, factory: e.ɵfac, providedIn: "root" })),
						e
					);
				})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(hO, [{ type: s1, args: [{ providedIn: "root" }] }], null, null);
				let hA = new iq(
						"undefined" == typeof ngDevMode || ngDevMode
							? "provideZoneChangeDetection token"
							: "",
					),
					hI = (() => {
						class e {}
						return (e.__NG_ELEMENT_ID__ = hP), e;
					})();
				function hP(e) {
					return (function (e, t, n) {
						if (ta(e) && !n) {
							let n = tQ(e.index, t);
							return new l$(n, n);
						}
						if (47 & e.type) {
							let e = t[15];
							return new l$(e, t);
						}
						return null;
					})(nr(), t9(), (16 & e) == 16);
				}
				class hT {
					supports(e) {
						return l5(e);
					}
					create(e) {
						return new hF(e);
					}
					constructor() {}
				}
				let hk = (e, t) => t;
				class hF {
					forEachItem(e) {
						let t;
						for (t = this._itHead; null !== t; t = t._next) e(t);
					}
					forEachOperation(e) {
						let t = this._itHead,
							n = this._removalsHead,
							r = 0,
							o = null;
						for (; t || n; ) {
							let i = !n || (t && t.currentIndex < h$(n, r, o)) ? t : n,
								s = h$(i, r, o),
								l = i.currentIndex;
							if (i === n) r--, (n = n._nextRemoved);
							else if (((t = t._next), null == i.previousIndex)) r++;
							else {
								!o && (o = []);
								let e = s - r,
									t = l - r;
								if (e != t) {
									for (let n = 0; n < e; n++) {
										let r = n < o.length ? o[n] : (o[n] = 0),
											i = r + n;
										t <= i && i < e && (o[n] = r + 1);
									}
									let n = i.previousIndex;
									o[n] = t - e;
								}
							}
							s !== l && e(i, s, l);
						}
					}
					forEachPreviousItem(e) {
						let t;
						for (t = this._previousItHead; null !== t; t = t._nextPrevious)
							e(t);
					}
					forEachAddedItem(e) {
						let t;
						for (t = this._additionsHead; null !== t; t = t._nextAdded) e(t);
					}
					forEachMovedItem(e) {
						let t;
						for (t = this._movesHead; null !== t; t = t._nextMoved) e(t);
					}
					forEachRemovedItem(e) {
						let t;
						for (t = this._removalsHead; null !== t; t = t._nextRemoved) e(t);
					}
					forEachIdentityChange(e) {
						let t;
						for (
							t = this._identityChangesHead;
							null !== t;
							t = t._nextIdentityChange
						)
							e(t);
					}
					diff(e) {
						if ((null == e && (e = []), !l5(e)))
							throw new O(
								900,
								ngDevMode &&
									`Error trying to diff '${j(
										e,
									)}'. Only arrays and iterables are allowed`,
							);
						return this.check(e) ? this : null;
					}
					onDestroy() {}
					check(e) {
						let t, n, r;
						this._reset();
						let o = this._itHead,
							i = !1;
						if (Array.isArray(e)) {
							this.length = e.length;
							for (let t = 0; t < this.length; t++)
								(n = e[t]),
									(r = this._trackByFn(t, n)),
									null !== o && Object.is(o.trackById, r)
										? (i && (o = this._verifyReinsertion(o, n, r, t)),
										  !Object.is(o.item, n) && this._addIdentityChange(o, n))
										: ((o = this._mismatch(o, n, r, t)), (i = !0)),
									(o = o._next);
						} else
							(t = 0),
								!(function (e, t) {
									if (Array.isArray(e))
										for (let n = 0; n < e.length; n++) t(e[n]);
									else {
										let n;
										let r = e[Symbol.iterator]();
										for (; !(n = r.next()).done; ) t(n.value);
									}
								})(e, (e) => {
									(r = this._trackByFn(t, e)),
										null !== o && Object.is(o.trackById, r)
											? (i && (o = this._verifyReinsertion(o, e, r, t)),
											  !Object.is(o.item, e) && this._addIdentityChange(o, e))
											: ((o = this._mismatch(o, e, r, t)), (i = !0)),
										(o = o._next),
										t++;
								}),
								(this.length = t);
						return this._truncate(o), (this.collection = e), this.isDirty;
					}
					get isDirty() {
						return (
							null !== this._additionsHead ||
							null !== this._movesHead ||
							null !== this._removalsHead ||
							null !== this._identityChangesHead
						);
					}
					_reset() {
						if (this.isDirty) {
							let e;
							for (
								e = this._previousItHead = this._itHead;
								null !== e;
								e = e._next
							)
								e._nextPrevious = e._next;
							for (e = this._additionsHead; null !== e; e = e._nextAdded)
								e.previousIndex = e.currentIndex;
							for (
								this._additionsHead = this._additionsTail = null,
									e = this._movesHead;
								null !== e;
								e = e._nextMoved
							)
								e.previousIndex = e.currentIndex;
							(this._movesHead = this._movesTail = null),
								(this._removalsHead = this._removalsTail = null),
								(this._identityChangesHead = this._identityChangesTail = null);
						}
					}
					_mismatch(e, t, n, r) {
						let o;
						return (
							null === e
								? (o = this._itTail)
								: ((o = e._prev), this._remove(e)),
							null !==
							(e =
								null === this._unlinkedRecords
									? null
									: this._unlinkedRecords.get(n, null))
								? (!Object.is(e.item, t) && this._addIdentityChange(e, t),
								  this._reinsertAfter(e, o, r))
								: null !==
								  (e =
										null === this._linkedRecords
											? null
											: this._linkedRecords.get(n, r))
								? (!Object.is(e.item, t) && this._addIdentityChange(e, t),
								  this._moveAfter(e, o, r))
								: (e = this._addAfter(new hR(t, n), o, r)),
							e
						);
					}
					_verifyReinsertion(e, t, n, r) {
						let o =
							null === this._unlinkedRecords
								? null
								: this._unlinkedRecords.get(n, null);
						return (
							null !== o
								? (e = this._reinsertAfter(o, e._prev, r))
								: e.currentIndex != r &&
								  ((e.currentIndex = r), this._addToMoves(e, r)),
							e
						);
					}
					_truncate(e) {
						for (; null !== e; ) {
							let t = e._next;
							this._addToRemovals(this._unlink(e)), (e = t);
						}
						null !== this._unlinkedRecords && this._unlinkedRecords.clear(),
							null !== this._additionsTail &&
								(this._additionsTail._nextAdded = null),
							null !== this._movesTail && (this._movesTail._nextMoved = null),
							null !== this._itTail && (this._itTail._next = null),
							null !== this._removalsTail &&
								(this._removalsTail._nextRemoved = null),
							null !== this._identityChangesTail &&
								(this._identityChangesTail._nextIdentityChange = null);
					}
					_reinsertAfter(e, t, n) {
						null !== this._unlinkedRecords && this._unlinkedRecords.remove(e);
						let r = e._prevRemoved,
							o = e._nextRemoved;
						return (
							null === r ? (this._removalsHead = o) : (r._nextRemoved = o),
							null === o ? (this._removalsTail = r) : (o._prevRemoved = r),
							this._insertAfter(e, t, n),
							this._addToMoves(e, n),
							e
						);
					}
					_moveAfter(e, t, n) {
						return (
							this._unlink(e),
							this._insertAfter(e, t, n),
							this._addToMoves(e, n),
							e
						);
					}
					_addAfter(e, t, n) {
						return (
							this._insertAfter(e, t, n),
							null === this._additionsTail
								? (this._additionsTail = this._additionsHead = e)
								: (this._additionsTail = this._additionsTail._nextAdded = e),
							e
						);
					}
					_insertAfter(e, t, n) {
						let r = null === t ? this._itHead : t._next;
						return (
							(e._next = r),
							(e._prev = t),
							null === r ? (this._itTail = e) : (r._prev = e),
							null === t ? (this._itHead = e) : (t._next = e),
							null === this._linkedRecords && (this._linkedRecords = new hL()),
							this._linkedRecords.put(e),
							(e.currentIndex = n),
							e
						);
					}
					_remove(e) {
						return this._addToRemovals(this._unlink(e));
					}
					_unlink(e) {
						null !== this._linkedRecords && this._linkedRecords.remove(e);
						let t = e._prev,
							n = e._next;
						return (
							null === t ? (this._itHead = n) : (t._next = n),
							null === n ? (this._itTail = t) : (n._prev = t),
							e
						);
					}
					_addToMoves(e, t) {
						return e.previousIndex === t
							? e
							: (null === this._movesTail
									? (this._movesTail = this._movesHead = e)
									: (this._movesTail = this._movesTail._nextMoved = e),
							  e);
					}
					_addToRemovals(e) {
						return (
							null === this._unlinkedRecords &&
								(this._unlinkedRecords = new hL()),
							this._unlinkedRecords.put(e),
							(e.currentIndex = null),
							(e._nextRemoved = null),
							null === this._removalsTail
								? ((this._removalsTail = this._removalsHead = e),
								  (e._prevRemoved = null))
								: ((e._prevRemoved = this._removalsTail),
								  (this._removalsTail = this._removalsTail._nextRemoved = e)),
							e
						);
					}
					_addIdentityChange(e, t) {
						return (
							(e.item = t),
							null === this._identityChangesTail
								? (this._identityChangesTail = this._identityChangesHead = e)
								: (this._identityChangesTail =
										this._identityChangesTail._nextIdentityChange =
											e),
							e
						);
					}
					constructor(e) {
						(this.length = 0),
							(this._linkedRecords = null),
							(this._unlinkedRecords = null),
							(this._previousItHead = null),
							(this._itHead = null),
							(this._itTail = null),
							(this._additionsHead = null),
							(this._additionsTail = null),
							(this._movesHead = null),
							(this._movesTail = null),
							(this._removalsHead = null),
							(this._removalsTail = null),
							(this._identityChangesHead = null),
							(this._identityChangesTail = null),
							(this._trackByFn = e || hk);
					}
				}
				class hR {
					constructor(e, t) {
						(this.item = e),
							(this.trackById = t),
							(this.currentIndex = null),
							(this.previousIndex = null),
							(this._nextPrevious = null),
							(this._prev = null),
							(this._next = null),
							(this._prevDup = null),
							(this._nextDup = null),
							(this._prevRemoved = null),
							(this._nextRemoved = null),
							(this._nextAdded = null),
							(this._nextMoved = null),
							(this._nextIdentityChange = null);
					}
				}
				class hN {
					add(e) {
						null === this._head
							? ((this._head = this._tail = e),
							  (e._nextDup = null),
							  (e._prevDup = null))
							: ((this._tail._nextDup = e),
							  (e._prevDup = this._tail),
							  (e._nextDup = null),
							  (this._tail = e));
					}
					get(e, t) {
						let n;
						for (n = this._head; null !== n; n = n._nextDup)
							if (
								(null === t || t <= n.currentIndex) &&
								Object.is(n.trackById, e)
							)
								return n;
						return null;
					}
					remove(e) {
						let t = e._prevDup,
							n = e._nextDup;
						return (
							null === t ? (this._head = n) : (t._nextDup = n),
							null === n ? (this._tail = t) : (n._prevDup = t),
							null === this._head
						);
					}
					constructor() {
						(this._head = null), (this._tail = null);
					}
				}
				class hL {
					put(e) {
						let t = e.trackById,
							n = this.map.get(t);
						!n && ((n = new hN()), this.map.set(t, n)), n.add(e);
					}
					get(e, t) {
						let n = this.map.get(e);
						return n ? n.get(e, t) : null;
					}
					remove(e) {
						let t = e.trackById,
							n = this.map.get(t);
						return n.remove(e) && this.map.delete(t), e;
					}
					get isEmpty() {
						return 0 === this.map.size;
					}
					clear() {
						this.map.clear();
					}
					constructor() {
						this.map = new Map();
					}
				}
				function h$(e, t, n) {
					let r = e.previousIndex;
					if (null === r) return r;
					let o = 0;
					return n && r < n.length && (o = n[r]), r + t + o;
				}
				class hV {
					supports(e) {
						return e instanceof Map || l2(e);
					}
					create() {
						return new hB();
					}
					constructor() {}
				}
				class hB {
					get isDirty() {
						return (
							null !== this._additionsHead ||
							null !== this._changesHead ||
							null !== this._removalsHead
						);
					}
					forEachItem(e) {
						let t;
						for (t = this._mapHead; null !== t; t = t._next) e(t);
					}
					forEachPreviousItem(e) {
						let t;
						for (t = this._previousMapHead; null !== t; t = t._nextPrevious)
							e(t);
					}
					forEachChangedItem(e) {
						let t;
						for (t = this._changesHead; null !== t; t = t._nextChanged) e(t);
					}
					forEachAddedItem(e) {
						let t;
						for (t = this._additionsHead; null !== t; t = t._nextAdded) e(t);
					}
					forEachRemovedItem(e) {
						let t;
						for (t = this._removalsHead; null !== t; t = t._nextRemoved) e(t);
					}
					diff(e) {
						if (e) {
							if (!(e instanceof Map || l2(e)))
								throw new O(
									900,
									ngDevMode &&
										`Error trying to diff '${j(
											e,
										)}'. Only maps and objects are allowed`,
								);
						} else e = new Map();
						return this.check(e) ? this : null;
					}
					onDestroy() {}
					check(e) {
						this._reset();
						let t = this._mapHead;
						if (
							((this._appendAfter = null),
							this._forEach(e, (e, n) => {
								if (t && t.key === n)
									this._maybeAddToChanges(t, e),
										(this._appendAfter = t),
										(t = t._next);
								else {
									let r = this._getOrCreateRecordForKey(n, e);
									t = this._insertBeforeOrAppend(t, r);
								}
							}),
							t)
						) {
							t._prev && (t._prev._next = null), (this._removalsHead = t);
							for (let e = t; null !== e; e = e._nextRemoved)
								e === this._mapHead && (this._mapHead = null),
									this._records.delete(e.key),
									(e._nextRemoved = e._next),
									(e.previousValue = e.currentValue),
									(e.currentValue = null),
									(e._prev = null),
									(e._next = null);
						}
						return (
							this._changesTail && (this._changesTail._nextChanged = null),
							this._additionsTail && (this._additionsTail._nextAdded = null),
							this.isDirty
						);
					}
					_insertBeforeOrAppend(e, t) {
						if (e) {
							let n = e._prev;
							return (
								(t._next = e),
								(t._prev = n),
								(e._prev = t),
								n && (n._next = t),
								e === this._mapHead && (this._mapHead = t),
								(this._appendAfter = e),
								e
							);
						}
						return (
							this._appendAfter
								? ((this._appendAfter._next = t), (t._prev = this._appendAfter))
								: (this._mapHead = t),
							(this._appendAfter = t),
							null
						);
					}
					_getOrCreateRecordForKey(e, t) {
						if (this._records.has(e)) {
							let n = this._records.get(e);
							this._maybeAddToChanges(n, t);
							let r = n._prev,
								o = n._next;
							return (
								r && (r._next = o),
								o && (o._prev = r),
								(n._next = null),
								(n._prev = null),
								n
							);
						}
						let n = new hU(e);
						return (
							this._records.set(e, n),
							(n.currentValue = t),
							this._addToAdditions(n),
							n
						);
					}
					_reset() {
						if (this.isDirty) {
							let e;
							for (
								this._previousMapHead = this._mapHead,
									e = this._previousMapHead;
								null !== e;
								e = e._next
							)
								e._nextPrevious = e._next;
							for (e = this._changesHead; null !== e; e = e._nextChanged)
								e.previousValue = e.currentValue;
							for (e = this._additionsHead; null != e; e = e._nextAdded)
								e.previousValue = e.currentValue;
							(this._changesHead = this._changesTail = null),
								(this._additionsHead = this._additionsTail = null),
								(this._removalsHead = null);
						}
					}
					_maybeAddToChanges(e, t) {
						!Object.is(t, e.currentValue) &&
							((e.previousValue = e.currentValue),
							(e.currentValue = t),
							this._addToChanges(e));
					}
					_addToAdditions(e) {
						null === this._additionsHead
							? (this._additionsHead = this._additionsTail = e)
							: ((this._additionsTail._nextAdded = e),
							  (this._additionsTail = e));
					}
					_addToChanges(e) {
						null === this._changesHead
							? (this._changesHead = this._changesTail = e)
							: ((this._changesTail._nextChanged = e), (this._changesTail = e));
					}
					_forEach(e, t) {
						e instanceof Map
							? e.forEach(t)
							: Object.keys(e).forEach((n) => t(e[n], n));
					}
					constructor() {
						(this._records = new Map()),
							(this._mapHead = null),
							(this._appendAfter = null),
							(this._previousMapHead = null),
							(this._changesHead = null),
							(this._changesTail = null),
							(this._additionsHead = null),
							(this._additionsTail = null),
							(this._removalsHead = null),
							(this._removalsTail = null);
					}
				}
				class hU {
					constructor(e) {
						(this.key = e),
							(this.previousValue = null),
							(this.currentValue = null),
							(this._nextPrevious = null),
							(this._next = null),
							(this._prev = null),
							(this._nextAdded = null),
							(this._nextRemoved = null),
							(this._nextChanged = null);
					}
				}
				function hH() {
					return new hz([new hT()]);
				}
				let hz = (() => {
					class e {
						static create(t, n) {
							if (null != n) {
								let e = n.factories.slice();
								t = t.concat(e);
							}
							return new e(t);
						}
						static extend(t) {
							return {
								provide: e,
								useFactory: (n) => e.create(t, n || hH()),
								deps: [[e, new rU(), new rV()]],
							};
						}
						find(e) {
							let t = this.factories.find((t) => t.supports(e));
							if (null != t) return t;
							throw new O(
								901,
								ngDevMode &&
									`Cannot find a differ supporting object '${e}' of type '${(function (
										e,
									) {
										return e.name || typeof e;
									})(e)}'`,
							);
						}
						constructor(e) {
							this.factories = e;
						}
					}
					return (
						(e.ɵprov = J({ token: e, providedIn: "root", factory: hH })), e
					);
				})();
				function hW() {
					return new hq([new hV()]);
				}
				let hq = (() => {
						class e {
							static create(t, n) {
								if (n) {
									let e = n.factories.slice();
									t = t.concat(e);
								}
								return new e(t);
							}
							static extend(t) {
								return {
									provide: e,
									useFactory: (n) => e.create(t, n || hW()),
									deps: [[e, new rU(), new rV()]],
								};
							}
							find(e) {
								let t = this.factories.find((t) => t.supports(e));
								if (t) return t;
								throw new O(
									901,
									ngDevMode && `Cannot find a differ supporting object '${e}'`,
								);
							}
							constructor(e) {
								this.factories = e;
							}
						}
						return (
							(e.ɵprov = J({ token: e, providedIn: "root", factory: hW })), e
						);
					})(),
					hG = [new hV()],
					hZ = [new hT()];
				new hz(hZ), new hq(hG);
				let hY = hj(null, "core", []),
					hQ = (() => {
						class e {
							constructor(e) {}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(e_(hw));
							}),
							(e.ɵmod = e1({ type: e })),
							(e.ɵinj = X({})),
							e
						);
					})();
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						hQ,
						[{ type: fQ }],
						function () {
							return [{ type: hw }];
						},
						null,
					);
				let hK = (() => {
					class e {
						get whenAllTasksComplete() {
							return (
								0 === this.collection.size && this.complete(), this.promise
							);
						}
						add() {
							if (this.completed) return -1;
							let e = this.taskId++;
							return this.collection.add(e), e;
						}
						remove(e) {
							!this.completed &&
								(this.collection.delete(e),
								0 === this.collection.size && this.complete());
						}
						ngOnDestroy() {
							this.complete(), this.collection.clear();
						}
						complete() {
							(this.completed = !0), this.resolve();
						}
						constructor() {
							(this.taskId = 0),
								(this.collection = new Set()),
								(this.ngZone = ex(hr)),
								(this.completed = !1),
								this.ngZone.runOutsideAngular(() => {
									this.promise = new Promise((e) => {
										this.resolve = e;
									});
								});
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = J({ token: e, factory: e.ɵfac, providedIn: "root" })),
						e
					);
				})();
				function hJ(e) {
					return "boolean" == typeof e ? e : null != e && "false" !== e;
				}
				function hX(e) {
					let t = e6(e);
					if (!t) return null;
					let n = new lz(t);
					return {
						get selector() {
							return n.selector;
						},
						get type() {
							return n.componentType;
						},
						get inputs() {
							return n.inputs;
						},
						get outputs() {
							return n.outputs;
						},
						get ngContentSelectors() {
							return n.ngContentSelectors;
						},
						get isStandalone() {
							return t.standalone;
						},
					};
				}
				("undefined" == typeof ngDevMode || ngDevMode) &&
					cE(
						hK,
						[{ type: s1, args: [{ providedIn: "root" }] }],
						function () {
							return [];
						},
						null,
					),
					"undefined" != typeof ngDevMode &&
						ngDevMode &&
						(ec.$localize =
							ec.$localize ||
							function () {
								throw Error(
									"It looks like your application or one of its dependencies is using i18n.\nAngular 9 introduced a global `$localize()` function that needs to be loaded.\nPlease run `ng add @angular/localize` from the Angular CLI.\n(For non-CLI projects, add `import '@angular/localize/init';` to your `polyfills.ts` file.\nFor server-side rendering applications add the import to your `main.server.ts` file.)",
								);
							});
			},
			"../../node_modules/@angular/platform-browser/fesm2022/platform-browser.mjs":
				function (e, t, n) {
					"use strict";
					let r;
					Object.defineProperty(t, "__esModule", { value: !0 });
					!(function (e, t) {
						for (var n in t)
							Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
					})(t, {
						BrowserModule: function () {
							return B;
						},
						Title: function () {
							return z;
						},
						platformBrowser: function () {
							return N;
						},
					});
					var o = n("../../node_modules/@swc/helpers/esm/_object_spread.js"),
						i = n(
							"../../node_modules/@swc/helpers/esm/_object_spread_props.js",
						),
						s = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs")),
						l = n("../../node_modules/@angular/common/fesm2022/common.mjs");
					n("../../node_modules/@angular/common/fesm2022/http.mjs");
					class a extends l.ɵDomAdapter {
						constructor() {
							super(...arguments), (this.supportsDOMEvents = !0);
						}
					}
					class u extends a {
						static makeCurrent() {
							(0, l.ɵsetRootDomAdapter)(new u());
						}
						onAndCancel(e, t, n) {
							return (
								e.addEventListener(t, n),
								() => {
									e.removeEventListener(t, n);
								}
							);
						}
						dispatchEvent(e, t) {
							e.dispatchEvent(t);
						}
						remove(e) {
							e.parentNode && e.parentNode.removeChild(e);
						}
						createElement(e, t) {
							return (t = t || this.getDefaultDocument()).createElement(e);
						}
						createHtmlDocument() {
							return document.implementation.createHTMLDocument("fakeTitle");
						}
						getDefaultDocument() {
							return document;
						}
						isElementNode(e) {
							return e.nodeType === Node.ELEMENT_NODE;
						}
						isShadowRoot(e) {
							return e instanceof DocumentFragment;
						}
						getGlobalEventTarget(e, t) {
							return "window" === t
								? window
								: "document" === t
								? e
								: "body" === t
								? e.body
								: null;
						}
						getBaseHref(e) {
							let t = (function () {
								return (d = d || document.querySelector("base"))
									? d.getAttribute("href")
									: null;
							})();
							return null == t
								? null
								: (function (e) {
										(r = r || document.createElement("a")).setAttribute(
											"href",
											e,
										);
										let t = r.pathname;
										return "/" === t.charAt(0) ? t : `/${t}`;
								  })(t);
						}
						resetBaseElement() {
							d = null;
						}
						getUserAgent() {
							return window.navigator.userAgent;
						}
						getCookie(e) {
							return (0, l.ɵparseCookieValue)(document.cookie, e);
						}
					}
					let d = null,
						c = (() => {
							class e {
								build() {
									return new XMLHttpRequest();
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)();
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let f = new s.InjectionToken("EventManagerPlugins"),
						h = (() => {
							class e {
								addEventListener(e, t, n) {
									let r = this._findPluginFor(t);
									return r.addEventListener(e, t, n);
								}
								getZone() {
									return this._zone;
								}
								_findPluginFor(e) {
									let t = this._eventNameToPlugin.get(e);
									if (t) return t;
									let n = this._plugins;
									for (let t = 0; t < n.length; t++) {
										let r = n[t];
										if (r.supports(e))
											return this._eventNameToPlugin.set(e, r), r;
									}
									throw Error(`No event manager plugin found for event ${e}`);
								}
								constructor(e, t) {
									(this._zone = t),
										(this._eventNameToPlugin = new Map()),
										e.forEach((e) => {
											e.manager = this;
										}),
										(this._plugins = e.slice().reverse());
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)(s.ɵɵinject(f), s.ɵɵinject(s.NgZone));
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					class p {
						constructor(e) {
							this._doc = e;
						}
					}
					let m = "ng-app-id",
						g = (() => {
							class e {
								addStyles(e) {
									for (let t of e) {
										let e = this.changeUsageCount(t, 1);
										1 === e && this.onStyleAdded(t);
									}
								}
								removeStyles(e) {
									for (let t of e) {
										let e = this.changeUsageCount(t, -1);
										e <= 0 && this.onStyleRemoved(t);
									}
								}
								ngOnDestroy() {
									let e = this.styleNodesInDOM;
									for (let t of (e && (e.forEach((e) => e.remove()), e.clear()),
									this.getAllStyles()))
										this.onStyleRemoved(t);
									this.resetHostNodes();
								}
								addHost(e) {
									for (let t of (this.hostNodes.add(e), this.getAllStyles()))
										this.addStyleToHost(e, t);
								}
								removeHost(e) {
									this.hostNodes.delete(e);
								}
								getAllStyles() {
									return this.styleRef.keys();
								}
								onStyleAdded(e) {
									for (let t of this.hostNodes) this.addStyleToHost(t, e);
								}
								onStyleRemoved(e) {
									var t, n;
									let r = this.styleRef;
									null === (t = r.get(e)) ||
										void 0 === t ||
										null === (n = t.elements) ||
										void 0 === n ||
										n.forEach((e) => e.remove()),
										r.delete(e);
								}
								collectServerRenderedStyles() {
									var e;
									let t =
										null === (e = this.doc.head) || void 0 === e
											? void 0
											: e.querySelectorAll(`style[${m}="${this.appId}"]`);
									if (null == t ? void 0 : t.length) {
										let e = new Map();
										return (
											t.forEach((t) => {
												null != t.textContent && e.set(t.textContent, t);
											}),
											e
										);
									}
									return null;
								}
								changeUsageCount(e, t) {
									let n = this.styleRef;
									if (n.has(e)) {
										let r = n.get(e);
										return (r.usage += t), r.usage;
									}
									return n.set(e, { usage: t, elements: [] }), t;
								}
								getStyleElement(e, t) {
									let n = this.styleNodesInDOM,
										r = null == n ? void 0 : n.get(t);
									if ((null == r ? void 0 : r.parentNode) === e)
										return (
											n.delete(t),
											r.removeAttribute(m),
											("undefined" == typeof ngDevMode || ngDevMode) &&
												r.setAttribute("ng-style-reused", ""),
											r
										);
									{
										let e = this.doc.createElement("style");
										return (
											this.nonce && e.setAttribute("nonce", this.nonce),
											(e.textContent = t),
											this.platformIsServer && e.setAttribute(m, this.appId),
											e
										);
									}
								}
								addStyleToHost(e, t) {
									var n;
									let r = this.getStyleElement(e, t);
									e.appendChild(r);
									let o = this.styleRef,
										i =
											null === (n = o.get(t)) || void 0 === n
												? void 0
												: n.elements;
									i ? i.push(r) : o.set(t, { elements: [r], usage: 1 });
								}
								resetHostNodes() {
									let e = this.hostNodes;
									e.clear(), e.add(this.doc.head);
								}
								constructor(e, t, n, r = {}) {
									(this.doc = e),
										(this.appId = t),
										(this.nonce = n),
										(this.platformId = r),
										(this.styleRef = new Map()),
										(this.hostNodes = new Set()),
										(this.styleNodesInDOM = this.collectServerRenderedStyles()),
										(this.platformIsServer = (0, l.isPlatformServer)(r)),
										this.resetHostNodes();
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)(
										s.ɵɵinject(l.DOCUMENT),
										s.ɵɵinject(s.APP_ID),
										s.ɵɵinject(s.CSP_NONCE, 8),
										s.ɵɵinject(s.PLATFORM_ID),
									);
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let v = {
							svg: "http://www.w3.org/2000/svg",
							xhtml: "http://www.w3.org/1999/xhtml",
							xlink: "http://www.w3.org/1999/xlink",
							xml: "http://www.w3.org/XML/1998/namespace",
							xmlns: "http://www.w3.org/2000/xmlns/",
							math: "http://www.w3.org/1998/MathML/",
						},
						y = /%COMP%/g,
						b = "%COMP%",
						_ = `_nghost-${b}`,
						j = `_ngcontent-${b}`,
						x = new s.InjectionToken("RemoveStylesOnCompDestory", {
							providedIn: "root",
							factory: () => !1,
						});
					function D(e, t) {
						return t.map((t) => t.replace(y, e));
					}
					let w = (() => {
						class e {
							createRenderer(e, t) {
								if (!e || !t) return this.defaultRenderer;
								this.platformIsServer &&
									t.encapsulation === s.ViewEncapsulation.ShadowDom &&
									(t = i._(o._({}, t), {
										encapsulation: s.ViewEncapsulation.Emulated,
									}));
								let n = this.getOrCreateRenderer(e, t);
								return (
									n instanceof A
										? n.applyToHost(e)
										: n instanceof O && n.applyStyles(),
									n
								);
							}
							getOrCreateRenderer(e, t) {
								let n = this.rendererByCompId,
									r = n.get(t.id);
								if (!r) {
									let o = this.doc,
										i = this.ngZone,
										l = this.eventManager,
										a = this.sharedStylesHost,
										u = this.removeStylesOnCompDestory,
										d = this.platformIsServer;
									switch (t.encapsulation) {
										case s.ViewEncapsulation.Emulated:
											r = new A(l, a, t, this.appId, u, o, i, d);
											break;
										case s.ViewEncapsulation.ShadowDom:
											return new E(l, a, e, t, o, i, this.nonce, d);
										default:
											r = new O(l, a, t, u, o, i, d);
									}
									(r.onDestroy = () => n.delete(t.id)), n.set(t.id, r);
								}
								return r;
							}
							ngOnDestroy() {
								this.rendererByCompId.clear();
							}
							constructor(e, t, n, r, o, i, s, a = null) {
								(this.eventManager = e),
									(this.sharedStylesHost = t),
									(this.appId = n),
									(this.removeStylesOnCompDestory = r),
									(this.doc = o),
									(this.platformId = i),
									(this.ngZone = s),
									(this.nonce = a),
									(this.rendererByCompId = new Map()),
									(this.platformIsServer = (0, l.isPlatformServer)(i)),
									(this.defaultRenderer = new M(
										e,
										o,
										s,
										this.platformIsServer,
									));
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(
									s.ɵɵinject(h),
									s.ɵɵinject(g),
									s.ɵɵinject(s.APP_ID),
									s.ɵɵinject(x),
									s.ɵɵinject(l.DOCUMENT),
									s.ɵɵinject(s.PLATFORM_ID),
									s.ɵɵinject(s.NgZone),
									s.ɵɵinject(s.CSP_NONCE),
								);
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode;
					class M {
						destroy() {}
						createElement(e, t) {
							return t
								? this.doc.createElementNS(v[t] || t, e)
								: this.doc.createElement(e);
						}
						createComment(e) {
							return this.doc.createComment(e);
						}
						createText(e) {
							return this.doc.createTextNode(e);
						}
						appendChild(e, t) {
							let n = S(e) ? e.content : e;
							n.appendChild(t);
						}
						insertBefore(e, t, n) {
							if (e) {
								let r = S(e) ? e.content : e;
								r.insertBefore(t, n);
							}
						}
						removeChild(e, t) {
							e && e.removeChild(t);
						}
						selectRootElement(e, t) {
							let n = "string" == typeof e ? this.doc.querySelector(e) : e;
							if (!n)
								throw Error(`The selector "${e}" did not match any elements`);
							return !t && (n.textContent = ""), n;
						}
						parentNode(e) {
							return e.parentNode;
						}
						nextSibling(e) {
							return e.nextSibling;
						}
						setAttribute(e, t, n, r) {
							if (r) {
								t = r + ":" + t;
								let o = v[r];
								o ? e.setAttributeNS(o, t, n) : e.setAttribute(t, n);
							} else e.setAttribute(t, n);
						}
						removeAttribute(e, t, n) {
							if (n) {
								let r = v[n];
								r ? e.removeAttributeNS(r, t) : e.removeAttribute(`${n}:${t}`);
							} else e.removeAttribute(t);
						}
						addClass(e, t) {
							e.classList.add(t);
						}
						removeClass(e, t) {
							e.classList.remove(t);
						}
						setStyle(e, t, n, r) {
							r &
							(s.RendererStyleFlags2.DashCase | s.RendererStyleFlags2.Important)
								? e.style.setProperty(
										t,
										n,
										r & s.RendererStyleFlags2.Important ? "important" : "",
								  )
								: (e.style[t] = n);
						}
						removeStyle(e, t, n) {
							n & s.RendererStyleFlags2.DashCase
								? e.style.removeProperty(t)
								: (e.style[t] = "");
						}
						setProperty(e, t, n) {
							("undefined" == typeof ngDevMode || ngDevMode) &&
								C(t, "property"),
								(e[t] = n);
						}
						setValue(e, t) {
							e.nodeValue = t;
						}
						listen(e, t, n) {
							if (
								(("undefined" == typeof ngDevMode || ngDevMode) &&
									C(t, "listener"),
								"string" == typeof e &&
									!(e = (0, l.ɵgetDOM)().getGlobalEventTarget(this.doc, e)))
							)
								throw Error(`Unsupported event target ${e} for event ${t}`);
							return this.eventManager.addEventListener(
								e,
								t,
								this.decoratePreventDefault(n),
							);
						}
						decoratePreventDefault(e) {
							return (t) => {
								if ("__ngUnwrap__" === t) return e;
								let n = this.platformIsServer
									? this.ngZone.runGuarded(() => e(t))
									: e(t);
								!1 === n && (t.preventDefault(), (t.returnValue = !1));
							};
						}
						constructor(e, t, n, r) {
							(this.eventManager = e),
								(this.doc = t),
								(this.ngZone = n),
								(this.platformIsServer = r),
								(this.data = Object.create(null)),
								(this.destroyNode = null);
						}
					}
					function C(e, t) {
						if (64 === e.charCodeAt(0))
							throw Error(`Unexpected synthetic ${t} ${e} found. Please make sure that:
  - Either \`BrowserAnimationsModule\` or \`NoopAnimationsModule\` are imported in your application.
  - There is corresponding configuration for the animation named \`${e}\` defined in the \`animations\` field of the \`@Component\` decorator (see https://angular.io/api/core/Component#animations).`);
					}
					function S(e) {
						return "TEMPLATE" === e.tagName && void 0 !== e.content;
					}
					class E extends M {
						nodeOrShadowRoot(e) {
							return e === this.hostEl ? this.shadowRoot : e;
						}
						appendChild(e, t) {
							return super.appendChild(this.nodeOrShadowRoot(e), t);
						}
						insertBefore(e, t, n) {
							return super.insertBefore(this.nodeOrShadowRoot(e), t, n);
						}
						removeChild(e, t) {
							return super.removeChild(this.nodeOrShadowRoot(e), t);
						}
						parentNode(e) {
							return this.nodeOrShadowRoot(
								super.parentNode(this.nodeOrShadowRoot(e)),
							);
						}
						destroy() {
							this.sharedStylesHost.removeHost(this.shadowRoot);
						}
						constructor(e, t, n, r, o, i, s, l) {
							super(e, o, i, l),
								(this.sharedStylesHost = t),
								(this.hostEl = n),
								(this.shadowRoot = n.attachShadow({ mode: "open" })),
								this.sharedStylesHost.addHost(this.shadowRoot);
							let a = D(r.id, r.styles);
							for (let e of a) {
								let t = document.createElement("style");
								s && t.setAttribute("nonce", s),
									(t.textContent = e),
									this.shadowRoot.appendChild(t);
							}
						}
					}
					class O extends M {
						applyStyles() {
							this.sharedStylesHost.addStyles(this.styles),
								this.rendererUsageCount++;
						}
						destroy() {
							if (
								this.removeStylesOnCompDestory &&
								(this.sharedStylesHost.removeStyles(this.styles),
								this.rendererUsageCount--,
								0 === this.rendererUsageCount)
							) {
								var e;
								null === (e = this.onDestroy) || void 0 === e || e.call(this);
							}
						}
						constructor(e, t, n, r, o, i, s, l) {
							super(e, o, i, s),
								(this.sharedStylesHost = t),
								(this.removeStylesOnCompDestory = r),
								(this.rendererUsageCount = 0),
								(this.styles = l ? D(l, n.styles) : n.styles);
						}
					}
					class A extends O {
						applyToHost(e) {
							this.applyStyles(), this.setAttribute(e, this.hostAttr, "");
						}
						createElement(e, t) {
							let n = super.createElement(e, t);
							return super.setAttribute(n, this.contentAttr, ""), n;
						}
						constructor(e, t, n, r, o, i, s, l) {
							var a, u;
							let d = r + "-" + n.id;
							super(e, t, n, o, i, s, l, d),
								(this.contentAttr = ((a = d), j.replace(y, a))),
								(this.hostAttr = ((u = d), _.replace(y, u)));
						}
					}
					let I = (() => {
						class e extends p {
							supports(e) {
								return !0;
							}
							addEventListener(e, t, n) {
								return (
									e.addEventListener(t, n, !1),
									() => this.removeEventListener(e, t, n)
								);
							}
							removeEventListener(e, t, n) {
								return e.removeEventListener(t, n);
							}
							constructor(e) {
								super(e);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(s.ɵɵinject(l.DOCUMENT));
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let P = ["alt", "control", "meta", "shift"],
						T = {
							"\b": "Backspace",
							"	": "Tab",
							"\x7f": "Delete",
							"\x1b": "Escape",
							Del: "Delete",
							Esc: "Escape",
							Left: "ArrowLeft",
							Right: "ArrowRight",
							Up: "ArrowUp",
							Down: "ArrowDown",
							Menu: "ContextMenu",
							Scroll: "ScrollLock",
							Win: "OS",
						},
						k = {
							alt: (e) => e.altKey,
							control: (e) => e.ctrlKey,
							meta: (e) => e.metaKey,
							shift: (e) => e.shiftKey,
						},
						F = (() => {
							class e extends p {
								supports(t) {
									return null != e.parseEventName(t);
								}
								addEventListener(t, n, r) {
									let o = e.parseEventName(n),
										i = e.eventCallback(o.fullKey, r, this.manager.getZone());
									return this.manager
										.getZone()
										.runOutsideAngular(() =>
											(0, l.ɵgetDOM)().onAndCancel(t, o.domEventName, i),
										);
								}
								static parseEventName(t) {
									let n = t.toLowerCase().split("."),
										r = n.shift();
									if (0 === n.length || !("keydown" === r || "keyup" === r))
										return null;
									let o = e._normalizeKey(n.pop()),
										i = "",
										s = n.indexOf("code");
									if (
										(s > -1 && (n.splice(s, 1), (i = "code.")),
										P.forEach((e) => {
											let t = n.indexOf(e);
											t > -1 && (n.splice(t, 1), (i += e + "."));
										}),
										(i += o),
										0 != n.length || 0 === o.length)
									)
										return null;
									let l = {};
									return (l.domEventName = r), (l.fullKey = i), l;
								}
								static matchEventFullKeyCode(e, t) {
									let n = T[e.key] || e.key,
										r = "";
									return (
										t.indexOf("code.") > -1 && ((n = e.code), (r = "code.")),
										null != n &&
											!!n &&
											(" " === (n = n.toLowerCase())
												? (n = "space")
												: "." === n && (n = "dot"),
											P.forEach((t) => {
												if (t !== n) {
													let n = k[t];
													n(e) && (r += t + ".");
												}
											}),
											(r += n) === t)
									);
								}
								static eventCallback(t, n, r) {
									return (o) => {
										e.matchEventFullKeyCode(o, t) && r.runGuarded(() => n(o));
									};
								}
								static _normalizeKey(e) {
									if ("esc" === e) return "escape";
									return e;
								}
								constructor(e) {
									super(e);
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)(s.ɵɵinject(l.DOCUMENT));
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let R = [
							{ provide: s.PLATFORM_ID, useValue: l.ɵPLATFORM_BROWSER_ID },
							{
								provide: s.PLATFORM_INITIALIZER,
								useValue: function () {
									u.makeCurrent();
								},
								multi: !0,
							},
							{
								provide: l.DOCUMENT,
								useFactory: function () {
									return (0, s.ɵsetDocument)(document), document;
								},
								deps: [],
							},
						],
						N = (0, s.createPlatformFactory)(s.platformCore, "browser", R),
						L = new s.InjectionToken(
							"undefined" == typeof ngDevMode || ngDevMode
								? "BrowserModule Providers Marker"
								: "",
						),
						$ = [
							{
								provide: s.ɵTESTABILITY_GETTER,
								useClass: class e {
									addToWindow(e) {
										(s.ɵglobal.getAngularTestability = (t, n = !0) => {
											let r = e.findTestabilityInTree(t, n);
											if (null == r)
												throw Error("Could not find testability for element.");
											return r;
										}),
											(s.ɵglobal.getAllAngularTestabilities = () =>
												e.getAllTestabilities()),
											(s.ɵglobal.getAllAngularRootElements = () =>
												e.getAllRootElements());
										!s.ɵglobal.frameworkStabilizers &&
											(s.ɵglobal.frameworkStabilizers = []),
											s.ɵglobal.frameworkStabilizers.push((e) => {
												let t = s.ɵglobal.getAllAngularTestabilities(),
													n = t.length,
													r = !1,
													o = function (t) {
														(r = r || t), 0 == --n && e(r);
													};
												t.forEach(function (e) {
													e.whenStable(o);
												});
											});
									}
									findTestabilityInTree(e, t, n) {
										if (null == t) return null;
										let r = e.getTestability(t);
										return null != r
											? r
											: n
											? (0, l.ɵgetDOM)().isShadowRoot(t)
												? this.findTestabilityInTree(e, t.host, !0)
												: this.findTestabilityInTree(e, t.parentElement, !0)
											: null;
									}
								},
								deps: [],
							},
							{
								provide: s.ɵTESTABILITY,
								useClass: s.Testability,
								deps: [s.NgZone, s.TestabilityRegistry, s.ɵTESTABILITY_GETTER],
							},
							{
								provide: s.Testability,
								useClass: s.Testability,
								deps: [s.NgZone, s.TestabilityRegistry, s.ɵTESTABILITY_GETTER],
							},
						],
						V = [
							{ provide: s.ɵINJECTOR_SCOPE, useValue: "root" },
							{
								provide: s.ErrorHandler,
								useFactory: function () {
									return new s.ErrorHandler();
								},
								deps: [],
							},
							{
								provide: f,
								useClass: I,
								multi: !0,
								deps: [l.DOCUMENT, s.NgZone, s.PLATFORM_ID],
							},
							{ provide: f, useClass: F, multi: !0, deps: [l.DOCUMENT] },
							w,
							g,
							h,
							{ provide: s.RendererFactory2, useExisting: w },
							{ provide: l.XhrFactory, useClass: c, deps: [] },
							"undefined" == typeof ngDevMode || ngDevMode
								? { provide: L, useValue: !0 }
								: [],
						],
						B = (() => {
							class e {
								static withServerTransition(t) {
									return {
										ngModule: e,
										providers: [{ provide: s.APP_ID, useValue: t.appId }],
									};
								}
								constructor(e) {
									if (("undefined" == typeof ngDevMode || ngDevMode) && e)
										throw Error(
											"Providers from the `BrowserModule` have already been loaded. If you need access to common directives such as NgIf and NgFor, import the `CommonModule` instead.",
										);
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)(s.ɵɵinject(L, 12));
								}),
								(e.ɵmod = s.ɵɵdefineNgModule({ type: e })),
								(e.ɵinj = s.ɵɵdefineInjector({
									providers: [...V, ...$],
									imports: [l.CommonModule, s.ApplicationModule],
								})),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let U = (() => {
						class e {
							addTag(e, t = !1) {
								return e ? this._getOrCreateElement(e, t) : null;
							}
							addTags(e, t = !1) {
								return e
									? e.reduce(
											(e, n) => (
												n && e.push(this._getOrCreateElement(n, t)), e
											),
											[],
									  )
									: [];
							}
							getTag(e) {
								return (e && this._doc.querySelector(`meta[${e}]`)) || null;
							}
							getTags(e) {
								if (!e) return [];
								let t = this._doc.querySelectorAll(`meta[${e}]`);
								return t ? [].slice.call(t) : [];
							}
							updateTag(e, t) {
								if (!e) return null;
								t = t || this._parseSelector(e);
								let n = this.getTag(t);
								return n
									? this._setMetaElementAttributes(e, n)
									: this._getOrCreateElement(e, !0);
							}
							removeTag(e) {
								this.removeTagElement(this.getTag(e));
							}
							removeTagElement(e) {
								e && this._dom.remove(e);
							}
							_getOrCreateElement(e, t = !1) {
								if (!t) {
									let t = this._parseSelector(e),
										n = this.getTags(t).filter((t) =>
											this._containsAttributes(e, t),
										)[0];
									if (void 0 !== n) return n;
								}
								let n = this._dom.createElement("meta");
								this._setMetaElementAttributes(e, n);
								let r = this._doc.getElementsByTagName("head")[0];
								return r.appendChild(n), n;
							}
							_setMetaElementAttributes(e, t) {
								return (
									Object.keys(e).forEach((n) =>
										t.setAttribute(this._getMetaKeyMap(n), e[n]),
									),
									t
								);
							}
							_parseSelector(e) {
								let t = e.name ? "name" : "property";
								return `${t}="${e[t]}"`;
							}
							_containsAttributes(e, t) {
								return Object.keys(e).every(
									(n) => t.getAttribute(this._getMetaKeyMap(n)) === e[n],
								);
							}
							_getMetaKeyMap(e) {
								return H[e] || e;
							}
							constructor(e) {
								(this._doc = e), (this._dom = (0, l.ɵgetDOM)());
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(s.ɵɵinject(l.DOCUMENT));
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({
								token: e,
								factory: function (e) {
									let t = null;
									return (t = e ? new e() : new U((0, s.ɵɵinject)(l.DOCUMENT)));
								},
								providedIn: "root",
							})),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let H = { httpEquiv: "http-equiv" },
						z = (() => {
							class e {
								getTitle() {
									return this._doc.title;
								}
								setTitle(e) {
									this._doc.title = e || "";
								}
								constructor(e) {
									this._doc = e;
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)(s.ɵɵinject(l.DOCUMENT));
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({
									token: e,
									factory: function (e) {
										let t = null;
										return (t = e
											? new e()
											: new z((0, s.ɵɵinject)(l.DOCUMENT)));
									},
									providedIn: "root",
								})),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode,
						"undefined" != typeof window && window;
					let W = {
							pan: !0,
							panstart: !0,
							panmove: !0,
							panend: !0,
							pancancel: !0,
							panleft: !0,
							panright: !0,
							panup: !0,
							pandown: !0,
							pinch: !0,
							pinchstart: !0,
							pinchmove: !0,
							pinchend: !0,
							pinchcancel: !0,
							pinchin: !0,
							pinchout: !0,
							press: !0,
							pressup: !0,
							rotate: !0,
							rotatestart: !0,
							rotatemove: !0,
							rotateend: !0,
							rotatecancel: !0,
							swipe: !0,
							swipeleft: !0,
							swiperight: !0,
							swipeup: !0,
							swipedown: !0,
							tap: !0,
							doubletap: !0,
						},
						q = new s.InjectionToken("HammerGestureConfig"),
						G = new s.InjectionToken("HammerLoader"),
						Z = (() => {
							class e {
								buildHammer(e) {
									let t = new Hammer(e, this.options);
									for (let e in (t.get("pinch").set({ enable: !0 }),
									t.get("rotate").set({ enable: !0 }),
									this.overrides))
										t.get(e).set(this.overrides[e]);
									return t;
								}
								constructor() {
									(this.events = []), (this.overrides = {});
								}
							}
							return (
								(e.ɵfac = function (t) {
									return new (t || e)();
								}),
								(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
								e
							);
						})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let Y = (() => {
						class e extends p {
							supports(e) {
								return (
									!!(
										W.hasOwnProperty(e.toLowerCase()) || this.isCustomEvent(e)
									) &&
									(!!window.Hammer ||
										!!this.loader ||
										(("undefined" == typeof ngDevMode || ngDevMode) &&
											this.console.warn(
												`The "${e}" event cannot be bound because Hammer.JS is not loaded and no custom loader has been specified.`,
											),
										!1))
								);
							}
							addEventListener(e, t, n) {
								let r = this.manager.getZone();
								if (((t = t.toLowerCase()), !window.Hammer && this.loader)) {
									this._loaderPromise =
										this._loaderPromise ||
										r.runOutsideAngular(() => this.loader());
									let o = !1,
										i = () => {
											o = !0;
										};
									return (
										r.runOutsideAngular(() =>
											this._loaderPromise
												.then(() => {
													if (!window.Hammer) {
														("undefined" == typeof ngDevMode || ngDevMode) &&
															this.console.warn(
																"The custom HAMMER_LOADER completed, but Hammer.JS is not present.",
															),
															(i = () => {});
														return;
													}
													!o && (i = this.addEventListener(e, t, n));
												})
												.catch(() => {
													("undefined" == typeof ngDevMode || ngDevMode) &&
														this.console.warn(
															`The "${t}" event cannot be bound because the custom Hammer.JS loader failed.`,
														),
														(i = () => {});
												}),
										),
										() => {
											i();
										}
									);
								}
								return r.runOutsideAngular(() => {
									let o = this._config.buildHammer(e),
										i = function (e) {
											r.runGuarded(function () {
												n(e);
											});
										};
									return (
										o.on(t, i),
										() => {
											o.off(t, i),
												"function" == typeof o.destroy && o.destroy();
										}
									);
								});
							}
							isCustomEvent(e) {
								return this._config.events.indexOf(e) > -1;
							}
							constructor(e, t, n, r) {
								super(e),
									(this._config = t),
									(this.console = n),
									(this.loader = r),
									(this._loaderPromise = null);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(
									s.ɵɵinject(l.DOCUMENT),
									s.ɵɵinject(q),
									s.ɵɵinject(s.ɵConsole),
									s.ɵɵinject(G, 8),
								);
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode,
						(() => {
							class e {}
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
								(e.ɵmod = s.ɵɵdefineNgModule({ type: e })),
								(e.ɵinj = s.ɵɵdefineInjector({
									providers: [
										{
											provide: f,
											useClass: Y,
											multi: !0,
											deps: [l.DOCUMENT, q, s.ɵConsole, [new s.Optional(), G]],
										},
										{ provide: q, useClass: Z, deps: [] },
									],
								}));
						})(),
						"undefined" == typeof ngDevMode || ngDevMode;
					let Q = (() => {
						class e {}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({
								token: e,
								factory: function (t) {
									let n = null;
									return (n = t ? new (t || e)() : s.ɵɵinject(K));
								},
								providedIn: "root",
							})),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode;
					let K = (() => {
						class e extends Q {
							sanitize(e, t) {
								if (null == t) return null;
								switch (e) {
									case s.SecurityContext.NONE:
										return t;
									case s.SecurityContext.HTML:
										if ((0, s.ɵallowSanitizationBypassAndThrow)(t, "HTML"))
											return (0, s.ɵunwrapSafeValue)(t);
										return (0, s.ɵ_sanitizeHtml)(
											this._doc,
											String(t),
										).toString();
									case s.SecurityContext.STYLE:
										if ((0, s.ɵallowSanitizationBypassAndThrow)(t, "Style"))
											return (0, s.ɵunwrapSafeValue)(t);
										return t;
									case s.SecurityContext.SCRIPT:
										if ((0, s.ɵallowSanitizationBypassAndThrow)(t, "Script"))
											return (0, s.ɵunwrapSafeValue)(t);
										throw Error("unsafe value used in a script context");
									case s.SecurityContext.URL:
										if ((0, s.ɵallowSanitizationBypassAndThrow)(t, "URL"))
											return (0, s.ɵunwrapSafeValue)(t);
										return (0, s.ɵ_sanitizeUrl)(String(t));
									case s.SecurityContext.RESOURCE_URL:
										if (
											(0, s.ɵallowSanitizationBypassAndThrow)(t, "ResourceURL")
										)
											return (0, s.ɵunwrapSafeValue)(t);
										throw Error(
											`unsafe value used in a resource URL context (see ${s.ɵXSS_SECURITY_URL})`,
										);
									default:
										throw Error(
											`Unexpected SecurityContext ${e} (see ${s.ɵXSS_SECURITY_URL})`,
										);
								}
							}
							bypassSecurityTrustHtml(e) {
								return (0, s.ɵbypassSanitizationTrustHtml)(e);
							}
							bypassSecurityTrustStyle(e) {
								return (0, s.ɵbypassSanitizationTrustStyle)(e);
							}
							bypassSecurityTrustScript(e) {
								return (0, s.ɵbypassSanitizationTrustScript)(e);
							}
							bypassSecurityTrustUrl(e) {
								return (0, s.ɵbypassSanitizationTrustUrl)(e);
							}
							bypassSecurityTrustResourceUrl(e) {
								return (0, s.ɵbypassSanitizationTrustResourceUrl)(e);
							}
							constructor(e) {
								super(), (this._doc = e);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)(s.ɵɵinject(l.DOCUMENT));
							}),
							(e.ɵprov = s.ɵɵdefineInjectable({
								token: e,
								factory: function (e) {
									let t = null;
									if (e) t = new e();
									else {
										var n;
										(n = s.ɵɵinject(s.Injector)),
											(t = new K(n.get(l.DOCUMENT)));
									}
									return t;
								},
								providedIn: "root",
							})),
							e
						);
					})();
					"undefined" == typeof ngDevMode || ngDevMode, new s.Version("16.0.0");
				},
			"../../node_modules/@angular/router/fesm2022/router.mjs": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					RouterModule: function () {
						return tJ;
					},
					RouterOutlet: function () {
						return e$;
					},
				});
				var r = n("../../node_modules/@swc/helpers/esm/_object_spread.js"),
					o = n("../../node_modules/@swc/helpers/esm/_object_spread_props.js"),
					i = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs")),
					s = n("../../node_modules/rxjs/dist/esm5/index.js"),
					l = n.ir(n("../../node_modules/@angular/common/fesm2022/common.mjs")),
					a = n("../../node_modules/rxjs/dist/esm5/operators/index.js"),
					u = n.ir(
						n(
							"../../node_modules/@angular/platform-browser/fesm2022/platform-browser.mjs",
						),
					);
				let d = "primary",
					c = Symbol("RouteTitle");
				class f {
					has(e) {
						return Object.prototype.hasOwnProperty.call(this.params, e);
					}
					get(e) {
						if (this.has(e)) {
							let t = this.params[e];
							return Array.isArray(t) ? t[0] : t;
						}
						return null;
					}
					getAll(e) {
						if (this.has(e)) {
							let t = this.params[e];
							return Array.isArray(t) ? t : [t];
						}
						return [];
					}
					get keys() {
						return Object.keys(this.params);
					}
					constructor(e) {
						this.params = e || {};
					}
				}
				function h(e) {
					return new f(e);
				}
				function p(e, t, n) {
					let r = n.path.split("/");
					if (
						r.length > e.length ||
						("full" === n.pathMatch && (t.hasChildren() || r.length < e.length))
					)
						return null;
					let o = {};
					for (let t = 0; t < r.length; t++) {
						let n = r[t],
							i = e[t],
							s = n.startsWith(":");
						if (s) o[n.substring(1)] = i;
						else if (n !== i.path) return null;
					}
					return { consumed: e.slice(0, r.length), posParams: o };
				}
				function m(e, t) {
					let n;
					let r = e ? Object.keys(e) : void 0,
						o = t ? Object.keys(t) : void 0;
					if (!r || !o || r.length != o.length) return !1;
					for (let o = 0; o < r.length; o++)
						if (!g(e[(n = r[o])], t[n])) return !1;
					return !0;
				}
				function g(e, t) {
					if (!(Array.isArray(e) && Array.isArray(t))) return e === t;
					{
						if (e.length !== t.length) return !1;
						let n = [...e].sort(),
							r = [...t].sort();
						return n.every((e, t) => r[t] === e);
					}
				}
				function v(e) {
					return e.length > 0 ? e[e.length - 1] : null;
				}
				function y(e) {
					return (0, s.isObservable)(e)
						? e
						: (0, i.ɵisPromise)(e)
						? (0, s.from)(Promise.resolve(e))
						: (0, s.of)(e);
				}
				let b = {
						exact: function e(t, n, r) {
							if (
								!S(t.segments, n.segments) ||
								!D(t.segments, n.segments, r) ||
								t.numberOfChildren !== n.numberOfChildren
							)
								return !1;
							for (let o in n.children)
								if (!t.children[o] || !e(t.children[o], n.children[o], r))
									return !1;
							return !0;
						},
						subset: x,
					},
					_ = {
						exact: function (e, t) {
							return m(e, t);
						},
						subset: function (e, t) {
							return (
								Object.keys(t).length <= Object.keys(e).length &&
								Object.keys(t).every((n) => g(e[n], t[n]))
							);
						},
						ignored: () => !0,
					};
				function j(e, t, n) {
					return (
						b[n.paths](e.root, t.root, n.matrixParams) &&
						_[n.queryParams](e.queryParams, t.queryParams) &&
						!("exact" === n.fragment && e.fragment !== t.fragment)
					);
				}
				function x(e, t, n) {
					return (function e(t, n, r, o) {
						if (t.segments.length > r.length) {
							let e = t.segments.slice(0, r.length);
							return !(!S(e, r) || n.hasChildren()) && !!D(e, r, o) && !0;
						}
						if (t.segments.length === r.length) {
							if (!S(t.segments, r) || !D(t.segments, r, o)) return !1;
							for (let e in n.children)
								if (!t.children[e] || !x(t.children[e], n.children[e], o))
									return !1;
							return !0;
						}
						{
							let i = r.slice(0, t.segments.length),
								s = r.slice(t.segments.length);
							return (
								!!(S(t.segments, i) && D(t.segments, i, o)) &&
								!!t.children[d] &&
								e(t.children[d], n, s, o)
							);
						}
					})(e, t, t.segments, n);
				}
				function D(e, t, n) {
					return t.every((t, r) => _[n](e[r].parameters, t.parameters));
				}
				class w {
					get queryParamMap() {
						return (
							!this._queryParamMap &&
								(this._queryParamMap = h(this.queryParams)),
							this._queryParamMap
						);
					}
					toString() {
						return A.serialize(this);
					}
					constructor(e = new M([], {}), t = {}, n = null) {
						if (
							((this.root = e),
							(this.queryParams = t),
							(this.fragment = n),
							("undefined" == typeof ngDevMode || ngDevMode) &&
								e.segments.length > 0)
						)
							throw new i.ɵRuntimeError(
								4015,
								"The root `UrlSegmentGroup` should not contain `segments`. Instead, these segments belong in the `children` so they can be associated with a named outlet.",
							);
					}
				}
				class M {
					hasChildren() {
						return this.numberOfChildren > 0;
					}
					get numberOfChildren() {
						return Object.keys(this.children).length;
					}
					toString() {
						return I(this);
					}
					constructor(e, t) {
						(this.segments = e),
							(this.children = t),
							(this.parent = null),
							Object.values(t).forEach((e) => (e.parent = this));
					}
				}
				class C {
					get parameterMap() {
						return (
							!this._parameterMap && (this._parameterMap = h(this.parameters)),
							this._parameterMap
						);
					}
					toString() {
						return N(this);
					}
					constructor(e, t) {
						(this.path = e), (this.parameters = t);
					}
				}
				function S(e, t) {
					return (
						e.length === t.length && e.every((e, n) => e.path === t[n].path)
					);
				}
				let E = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function () {
								return new O();
							},
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				class O {
					parse(e) {
						let t = new H(e);
						return new w(
							t.parseRootSegment(),
							t.parseQueryParams(),
							t.parseFragment(),
						);
					}
					serialize(e) {
						let t = `/${(function e(t, n) {
								if (!t.hasChildren()) return I(t);
								if (n) {
									let n = t.children[d] ? e(t.children[d], !1) : "",
										r = [];
									return (
										Object.entries(t.children).forEach(([t, n]) => {
											t !== d && r.push(`${t}:${e(n, !1)}`);
										}),
										r.length > 0 ? `${n}(${r.join("//")})` : n
									);
								}
								{
									var r, o;
									let n;
									let i =
										((r = t),
										(o = (n, r) =>
											r === d ? [e(t.children[d], !1)] : [`${r}:${e(n, !1)}`]),
										(n = []),
										Object.entries(r.children).forEach(([e, t]) => {
											e === d && (n = n.concat(o(t, e)));
										}),
										Object.entries(r.children).forEach(([e, t]) => {
											e !== d && (n = n.concat(o(t, e)));
										}),
										n);
									return 1 === Object.keys(t.children).length &&
										null != t.children[d]
										? `${I(t)}/${i[0]}`
										: `${I(t)}/(${i.join("//")})`;
								}
							})(e.root, !0)}`,
							n = (function (e) {
								let t = Object.keys(e)
									.map((t) => {
										let n = e[t];
										return Array.isArray(n)
											? n.map((e) => `${T(t)}=${T(e)}`).join("&")
											: `${T(t)}=${T(n)}`;
									})
									.filter((e) => !!e);
								return t.length ? `?${t.join("&")}` : "";
							})(e.queryParams),
							r =
								"string" == typeof e.fragment
									? `#${(function (e) {
											return encodeURI(e);
									  })(e.fragment)}`
									: "";
						return `${t}${n}${r}`;
					}
				}
				let A = new O();
				function I(e) {
					return e.segments.map((e) => N(e)).join("/");
				}
				function P(e) {
					return encodeURIComponent(e)
						.replace(/%40/g, "@")
						.replace(/%3A/gi, ":")
						.replace(/%24/g, "$")
						.replace(/%2C/gi, ",");
				}
				function T(e) {
					return P(e).replace(/%3B/gi, ";");
				}
				function k(e) {
					return P(e)
						.replace(/\(/g, "%28")
						.replace(/\)/g, "%29")
						.replace(/%26/gi, "&");
				}
				function F(e) {
					return decodeURIComponent(e);
				}
				function R(e) {
					return F(e.replace(/\+/g, "%20"));
				}
				function N(e) {
					return `${k(e.path)}${(function (e) {
						return Object.keys(e)
							.map((t) => `;${k(t)}=${k(e[t])}`)
							.join("");
					})(e.parameters)}`;
				}
				let L = /^[^\/()?;#]+/;
				function $(e) {
					let t = e.match(L);
					return t ? t[0] : "";
				}
				let V = /^[^\/()?;=#]+/,
					B = /^[^=?&#]+/,
					U = /^[^&#]+/;
				class H {
					parseRootSegment() {
						return (this.consumeOptional("/"),
						"" === this.remaining ||
							this.peekStartsWith("?") ||
							this.peekStartsWith("#"))
							? new M([], {})
							: new M([], this.parseChildren());
					}
					parseQueryParams() {
						let e = {};
						if (this.consumeOptional("?"))
							do this.parseQueryParam(e);
							while (this.consumeOptional("&"));
						return e;
					}
					parseFragment() {
						return this.consumeOptional("#")
							? decodeURIComponent(this.remaining)
							: null;
					}
					parseChildren() {
						if ("" === this.remaining) return {};
						this.consumeOptional("/");
						let e = [];
						for (
							!this.peekStartsWith("(") && e.push(this.parseSegment());
							this.peekStartsWith("/") &&
							!this.peekStartsWith("//") &&
							!this.peekStartsWith("/(");
						)
							this.capture("/"), e.push(this.parseSegment());
						let t = {};
						this.peekStartsWith("/(") &&
							(this.capture("/"), (t = this.parseParens(!0)));
						let n = {};
						return (
							this.peekStartsWith("(") && (n = this.parseParens(!1)),
							(e.length > 0 || Object.keys(t).length > 0) &&
								(n[d] = new M(e, t)),
							n
						);
					}
					parseSegment() {
						let e = $(this.remaining);
						if ("" === e && this.peekStartsWith(";"))
							throw new i.ɵRuntimeError(
								4009,
								("undefined" == typeof ngDevMode || ngDevMode) &&
									`Empty path url segment cannot have parameters: '${this.remaining}'.`,
							);
						return this.capture(e), new C(F(e), this.parseMatrixParams());
					}
					parseMatrixParams() {
						let e = {};
						for (; this.consumeOptional(";"); ) this.parseParam(e);
						return e;
					}
					parseParam(e) {
						let t = (function (e) {
							let t = e.match(V);
							return t ? t[0] : "";
						})(this.remaining);
						if (!t) return;
						this.capture(t);
						let n = "";
						if (this.consumeOptional("=")) {
							let e = $(this.remaining);
							e && ((n = e), this.capture(n));
						}
						e[F(t)] = F(n);
					}
					parseQueryParam(e) {
						let t = (function (e) {
							let t = e.match(B);
							return t ? t[0] : "";
						})(this.remaining);
						if (!t) return;
						this.capture(t);
						let n = "";
						if (this.consumeOptional("=")) {
							let e = (function (e) {
								let t = e.match(U);
								return t ? t[0] : "";
							})(this.remaining);
							e && ((n = e), this.capture(n));
						}
						let r = R(t),
							o = R(n);
						if (e.hasOwnProperty(r)) {
							let t = e[r];
							!Array.isArray(t) && ((t = [t]), (e[r] = t)), t.push(o);
						} else e[r] = o;
					}
					parseParens(e) {
						let t = {};
						for (
							this.capture("(");
							!this.consumeOptional(")") && this.remaining.length > 0;
						) {
							let n,
								r = $(this.remaining),
								o = this.remaining[r.length];
							if ("/" !== o && ")" !== o && ";" !== o)
								throw new i.ɵRuntimeError(
									4010,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										`Cannot parse url '${this.url}'`,
								);
							r.indexOf(":") > -1
								? ((n = r.slice(0, r.indexOf(":"))),
								  this.capture(n),
								  this.capture(":"))
								: e && (n = d);
							let s = this.parseChildren();
							(t[n] = 1 === Object.keys(s).length ? s[d] : new M([], s)),
								this.consumeOptional("//");
						}
						return t;
					}
					peekStartsWith(e) {
						return this.remaining.startsWith(e);
					}
					consumeOptional(e) {
						return (
							!!this.peekStartsWith(e) &&
							((this.remaining = this.remaining.substring(e.length)), !0)
						);
					}
					capture(e) {
						if (!this.consumeOptional(e))
							throw new i.ɵRuntimeError(
								4011,
								("undefined" == typeof ngDevMode || ngDevMode) &&
									`Expected "${e}".`,
							);
					}
					constructor(e) {
						(this.url = e), (this.remaining = e);
					}
				}
				function z(e) {
					return e.segments.length > 0 ? new M([], { [d]: e }) : e;
				}
				function W(e) {
					return e instanceof w;
				}
				function q(e) {
					let t;
					let n = (function n(r) {
							let o = {};
							for (let e of r.children) {
								let t = n(e);
								o[e.outlet] = t;
							}
							let i = new M(r.url, o);
							return r === e && (t = i), i;
						})(e.root),
						r = z(n);
					return null != t ? t : r;
				}
				function G(e, t, n, r) {
					let o = e;
					for (; o.parent; ) o = o.parent;
					if (0 === t.length) return Q(o, o, o, n, r);
					let s = (function (e) {
						if ("string" == typeof e[0] && 1 === e.length && "/" === e[0])
							return new K(!0, 0, e);
						let t = 0,
							n = !1,
							r = e.reduce((e, r, o) => {
								if ("object" == typeof r && null != r) {
									if (r.outlets) {
										let t = {};
										return (
											Object.entries(r.outlets).forEach(([e, n]) => {
												t[e] = "string" == typeof n ? n.split("/") : n;
											}),
											[...e, { outlets: t }]
										);
									}
									if (r.segmentPath) return [...e, r.segmentPath];
								}
								return "string" != typeof r
									? [...e, r]
									: 0 === o
									? (r.split("/").forEach((r, o) => {
											(0 == o && "." === r) ||
												(0 == o && "" === r
													? (n = !0)
													: ".." === r
													? t++
													: "" != r && e.push(r));
									  }),
									  e)
									: [...e, r];
							}, []);
						return new K(n, t, r);
					})(t);
					if (s.toRoot()) return Q(o, o, new M([], {}), n, r);
					let l = (function (e, t, n) {
							if (e.isAbsolute) return new J(t, !0, 0);
							if (!n) return new J(t, !1, NaN);
							if (null === n.parent) return new J(n, !0, 0);
							let r = Z(e.commands[0]) ? 0 : 1,
								o = n.segments.length - 1 + r;
							return (function (e, t, n) {
								let r = e,
									o = t,
									s = n;
								for (; s > o; ) {
									if (((s -= o), !(r = r.parent)))
										throw new i.ɵRuntimeError(
											4005,
											("undefined" == typeof ngDevMode || ngDevMode) &&
												"Invalid number of '../'",
										);
									o = r.segments.length;
								}
								return new J(r, !1, o - s);
							})(n, o, e.numberOfDoubleDots);
						})(s, o, e),
						a = l.processChildren
							? ee(l.segmentGroup, l.index, s.commands)
							: X(l.segmentGroup, l.index, s.commands);
					return Q(o, l.segmentGroup, a, n, r);
				}
				function Z(e) {
					return (
						"object" == typeof e && null != e && !e.outlets && !e.segmentPath
					);
				}
				function Y(e) {
					return "object" == typeof e && null != e && e.outlets;
				}
				function Q(e, t, n, r, o) {
					let i,
						s = {};
					r &&
						Object.entries(r).forEach(([e, t]) => {
							s[e] = Array.isArray(t) ? t.map((e) => `${e}`) : `${t}`;
						}),
						(i =
							e === t
								? n
								: (function e(t, n, r) {
										let o = {};
										return (
											Object.entries(t.children).forEach(([t, i]) => {
												i === n ? (o[t] = r) : (o[t] = e(i, n, r));
											}),
											new M(t.segments, o)
										);
								  })(e, t, n));
					let l = z(
						(function e(t) {
							let n = {};
							for (let r of Object.keys(t.children)) {
								let o = t.children[r],
									i = e(o);
								if (r === d && 0 === i.segments.length && i.hasChildren())
									for (let [e, t] of Object.entries(i.children)) n[e] = t;
								else (i.segments.length > 0 || i.hasChildren()) && (n[r] = i);
							}
							let r = new M(t.segments, n);
							return (function (e) {
								if (1 === e.numberOfChildren && e.children[d]) {
									let t = e.children[d];
									return new M(e.segments.concat(t.segments), t.children);
								}
								return e;
							})(r);
						})(i),
					);
					return new w(l, s, o);
				}
				class K {
					toRoot() {
						return (
							this.isAbsolute &&
							1 === this.commands.length &&
							"/" == this.commands[0]
						);
					}
					constructor(e, t, n) {
						if (
							((this.isAbsolute = e),
							(this.numberOfDoubleDots = t),
							(this.commands = n),
							e && n.length > 0 && Z(n[0]))
						)
							throw new i.ɵRuntimeError(
								4003,
								("undefined" == typeof ngDevMode || ngDevMode) &&
									"Root segment cannot have matrix parameters",
							);
						let r = n.find(Y);
						if (r && r !== v(n))
							throw new i.ɵRuntimeError(
								4004,
								("undefined" == typeof ngDevMode || ngDevMode) &&
									"{outlets:{}} has to be the last command",
							);
					}
				}
				class J {
					constructor(e, t, n) {
						(this.segmentGroup = e),
							(this.processChildren = t),
							(this.index = n);
					}
				}
				function X(e, t, n) {
					if (
						(!e && (e = new M([], {})),
						0 === e.segments.length && e.hasChildren())
					)
						return ee(e, t, n);
					let r = (function (e, t, n) {
							let r = 0,
								o = t,
								i = { match: !1, pathIndex: 0, commandIndex: 0 };
							for (; o < e.segments.length; ) {
								if (r >= n.length) return i;
								let t = e.segments[o],
									s = n[r];
								if (Y(s)) break;
								let l = `${s}`,
									a = r < n.length - 1 ? n[r + 1] : null;
								if (o > 0 && void 0 === l) break;
								if (l && a && "object" == typeof a && void 0 === a.outlets) {
									if (!er(l, a, t)) return i;
									r += 2;
								} else {
									if (!er(l, {}, t)) return i;
									r++;
								}
								o++;
							}
							return { match: !0, pathIndex: o, commandIndex: r };
						})(e, t, n),
						o = n.slice(r.commandIndex);
					if (r.match && r.pathIndex < e.segments.length) {
						let t = new M(e.segments.slice(0, r.pathIndex), {});
						return (
							(t.children[d] = new M(
								e.segments.slice(r.pathIndex),
								e.children,
							)),
							ee(t, 0, o)
						);
					}
					if (r.match && 0 === o.length) return new M(e.segments, {});
					if (r.match && !e.hasChildren()) return et(e, t, n);
					else if (r.match) return ee(e, 0, o);
					else return et(e, t, n);
				}
				function ee(e, t, n) {
					if (0 === n.length) return new M(e.segments, {});
					{
						var r;
						let o = Y((r = n)[0]) ? r[0].outlets : { [d]: r },
							i = {};
						if (
							!o[d] &&
							e.children[d] &&
							1 === e.numberOfChildren &&
							0 === e.children[d].segments.length
						) {
							let r = ee(e.children[d], t, n);
							return new M(e.segments, r.children);
						}
						return (
							Object.entries(o).forEach(([n, r]) => {
								"string" == typeof r && (r = [r]),
									null !== r && (i[n] = X(e.children[n], t, r));
							}),
							Object.entries(e.children).forEach(([e, t]) => {
								void 0 === o[e] && (i[e] = t);
							}),
							new M(e.segments, i)
						);
					}
				}
				function et(e, t, n) {
					let r = e.segments.slice(0, t),
						o = 0;
					for (; o < n.length; ) {
						let i = n[o];
						if (Y(i)) {
							let e = (function (e) {
								let t = {};
								return (
									Object.entries(e).forEach(([e, n]) => {
										"string" == typeof n && (n = [n]),
											null !== n && (t[e] = et(new M([], {}), 0, n));
									}),
									t
								);
							})(i.outlets);
							return new M(r, e);
						}
						if (0 === o && Z(n[0])) {
							let i = e.segments[t];
							r.push(new C(i.path, en(n[0]))), o++;
							continue;
						}
						let s = Y(i) ? i.outlets[d] : `${i}`,
							l = o < n.length - 1 ? n[o + 1] : null;
						s && l && Z(l)
							? (r.push(new C(s, en(l))), (o += 2))
							: (r.push(new C(s, {})), o++);
					}
					return new M(r, {});
				}
				function en(e) {
					let t = {};
					return Object.entries(e).forEach(([e, n]) => (t[e] = `${n}`)), t;
				}
				function er(e, t, n) {
					return e == n.path && m(t, n.parameters);
				}
				let eo = "imperative";
				class ei {
					constructor(e, t) {
						(this.id = e), (this.url = t);
					}
				}
				class es extends ei {
					toString() {
						return `NavigationStart(id: ${this.id}, url: '${this.url}')`;
					}
					constructor(e, t, n = "imperative", r = null) {
						super(e, t),
							(this.type = 0),
							(this.navigationTrigger = n),
							(this.restoredState = r);
					}
				}
				class el extends ei {
					toString() {
						return `NavigationEnd(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}')`;
					}
					constructor(e, t, n) {
						super(e, t), (this.urlAfterRedirects = n), (this.type = 1);
					}
				}
				class ea extends ei {
					toString() {
						return `NavigationCancel(id: ${this.id}, url: '${this.url}')`;
					}
					constructor(e, t, n, r) {
						super(e, t), (this.reason = n), (this.code = r), (this.type = 2);
					}
				}
				class eu extends ei {
					constructor(e, t, n, r) {
						super(e, t), (this.reason = n), (this.code = r), (this.type = 16);
					}
				}
				class ed extends ei {
					toString() {
						return `NavigationError(id: ${this.id}, url: '${this.url}', error: ${this.error})`;
					}
					constructor(e, t, n, r) {
						super(e, t), (this.error = n), (this.target = r), (this.type = 3);
					}
				}
				class ec extends ei {
					toString() {
						return `RoutesRecognized(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}', state: ${this.state})`;
					}
					constructor(e, t, n, r) {
						super(e, t),
							(this.urlAfterRedirects = n),
							(this.state = r),
							(this.type = 4);
					}
				}
				class ef extends ei {
					toString() {
						return `GuardsCheckStart(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}', state: ${this.state})`;
					}
					constructor(e, t, n, r) {
						super(e, t),
							(this.urlAfterRedirects = n),
							(this.state = r),
							(this.type = 7);
					}
				}
				class eh extends ei {
					toString() {
						return `GuardsCheckEnd(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}', state: ${this.state}, shouldActivate: ${this.shouldActivate})`;
					}
					constructor(e, t, n, r, o) {
						super(e, t),
							(this.urlAfterRedirects = n),
							(this.state = r),
							(this.shouldActivate = o),
							(this.type = 8);
					}
				}
				class ep extends ei {
					toString() {
						return `ResolveStart(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}', state: ${this.state})`;
					}
					constructor(e, t, n, r) {
						super(e, t),
							(this.urlAfterRedirects = n),
							(this.state = r),
							(this.type = 5);
					}
				}
				class em extends ei {
					toString() {
						return `ResolveEnd(id: ${this.id}, url: '${this.url}', urlAfterRedirects: '${this.urlAfterRedirects}', state: ${this.state})`;
					}
					constructor(e, t, n, r) {
						super(e, t),
							(this.urlAfterRedirects = n),
							(this.state = r),
							(this.type = 6);
					}
				}
				class eg {
					toString() {
						return `RouteConfigLoadStart(path: ${this.route.path})`;
					}
					constructor(e) {
						(this.route = e), (this.type = 9);
					}
				}
				class ev {
					toString() {
						return `RouteConfigLoadEnd(path: ${this.route.path})`;
					}
					constructor(e) {
						(this.route = e), (this.type = 10);
					}
				}
				class ey {
					toString() {
						let e =
							(this.snapshot.routeConfig && this.snapshot.routeConfig.path) ||
							"";
						return `ChildActivationStart(path: '${e}')`;
					}
					constructor(e) {
						(this.snapshot = e), (this.type = 11);
					}
				}
				class eb {
					toString() {
						let e =
							(this.snapshot.routeConfig && this.snapshot.routeConfig.path) ||
							"";
						return `ChildActivationEnd(path: '${e}')`;
					}
					constructor(e) {
						(this.snapshot = e), (this.type = 12);
					}
				}
				class e_ {
					toString() {
						let e =
							(this.snapshot.routeConfig && this.snapshot.routeConfig.path) ||
							"";
						return `ActivationStart(path: '${e}')`;
					}
					constructor(e) {
						(this.snapshot = e), (this.type = 13);
					}
				}
				class ej {
					toString() {
						let e =
							(this.snapshot.routeConfig && this.snapshot.routeConfig.path) ||
							"";
						return `ActivationEnd(path: '${e}')`;
					}
					constructor(e) {
						(this.snapshot = e), (this.type = 14);
					}
				}
				class ex {
					toString() {
						let e = this.position
							? `${this.position[0]}, ${this.position[1]}`
							: null;
						return `Scroll(anchor: '${this.anchor}', position: '${e}')`;
					}
					constructor(e, t, n) {
						(this.routerEvent = e),
							(this.position = t),
							(this.anchor = n),
							(this.type = 15);
					}
				}
				class eD {
					constructor() {
						(this.outlet = null),
							(this.route = null),
							(this.injector = null),
							(this.children = new ew()),
							(this.attachRef = null);
					}
				}
				let ew = (() => {
					class e {
						onChildOutletCreated(e, t) {
							let n = this.getOrCreateContext(e);
							(n.outlet = t), this.contexts.set(e, n);
						}
						onChildOutletDestroyed(e) {
							let t = this.getContext(e);
							t && ((t.outlet = null), (t.attachRef = null));
						}
						onOutletDeactivated() {
							let e = this.contexts;
							return (this.contexts = new Map()), e;
						}
						onOutletReAttached(e) {
							this.contexts = e;
						}
						getOrCreateContext(e) {
							let t = this.getContext(e);
							return !t && ((t = new eD()), this.contexts.set(e, t)), t;
						}
						getContext(e) {
							return this.contexts.get(e) || null;
						}
						constructor() {
							this.contexts = new Map();
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				class eM {
					get root() {
						return this._root.value;
					}
					parent(e) {
						let t = this.pathFromRoot(e);
						return t.length > 1 ? t[t.length - 2] : null;
					}
					children(e) {
						let t = eC(e, this._root);
						return t ? t.children.map((e) => e.value) : [];
					}
					firstChild(e) {
						let t = eC(e, this._root);
						return t && t.children.length > 0 ? t.children[0].value : null;
					}
					siblings(e) {
						let t = eS(e, this._root);
						if (t.length < 2) return [];
						let n = t[t.length - 2].children.map((e) => e.value);
						return n.filter((t) => t !== e);
					}
					pathFromRoot(e) {
						return eS(e, this._root).map((e) => e.value);
					}
					constructor(e) {
						this._root = e;
					}
				}
				function eC(e, t) {
					if (e === t.value) return t;
					for (let n of t.children) {
						let t = eC(e, n);
						if (t) return t;
					}
					return null;
				}
				function eS(e, t) {
					if (e === t.value) return [t];
					for (let n of t.children) {
						let r = eS(e, n);
						if (r.length) return r.unshift(t), r;
					}
					return [];
				}
				class eE {
					toString() {
						return `TreeNode(${this.value})`;
					}
					constructor(e, t) {
						(this.value = e), (this.children = t);
					}
				}
				function eO(e) {
					let t = {};
					return e && e.children.forEach((e) => (t[e.value.outlet] = e)), t;
				}
				class eA extends eM {
					toString() {
						return this.snapshot.toString();
					}
					constructor(e, t) {
						super(e), (this.snapshot = t), eR(this, e);
					}
				}
				function eI(e, t) {
					let n = (function (e, t) {
							let n = new ek([], {}, {}, "", {}, d, t, null, {});
							return new eF("", new eE(n, []));
						})(e, t),
						r = new s.BehaviorSubject([new C("", {})]),
						o = new s.BehaviorSubject({}),
						i = new s.BehaviorSubject({}),
						l = new s.BehaviorSubject({}),
						a = new s.BehaviorSubject(""),
						u = new eP(r, o, l, a, i, d, t, n.root);
					return (u.snapshot = n.root), new eA(new eE(u, []), n);
				}
				class eP {
					get routeConfig() {
						return this._futureSnapshot.routeConfig;
					}
					get root() {
						return this._routerState.root;
					}
					get parent() {
						return this._routerState.parent(this);
					}
					get firstChild() {
						return this._routerState.firstChild(this);
					}
					get children() {
						return this._routerState.children(this);
					}
					get pathFromRoot() {
						return this._routerState.pathFromRoot(this);
					}
					get paramMap() {
						return (
							!this._paramMap &&
								(this._paramMap = this.params.pipe((0, a.map)((e) => h(e)))),
							this._paramMap
						);
					}
					get queryParamMap() {
						return (
							!this._queryParamMap &&
								(this._queryParamMap = this.queryParams.pipe(
									(0, a.map)((e) => h(e)),
								)),
							this._queryParamMap
						);
					}
					toString() {
						return this.snapshot
							? this.snapshot.toString()
							: `Future(${this._futureSnapshot})`;
					}
					constructor(e, t, n, r, o, i, l, u) {
						var d, f;
						(this.urlSubject = e),
							(this.paramsSubject = t),
							(this.queryParamsSubject = n),
							(this.fragmentSubject = r),
							(this.dataSubject = o),
							(this.outlet = i),
							(this.component = l),
							(this._futureSnapshot = u),
							(this.title =
								null !==
									(f =
										null === (d = this.dataSubject) || void 0 === d
											? void 0
											: d.pipe((0, a.map)((e) => e[c]))) && void 0 !== f
									? f
									: (0, s.of)(void 0)),
							(this.url = e),
							(this.params = t),
							(this.queryParams = n),
							(this.fragment = r),
							(this.data = o);
					}
				}
				function eT(e, t = "emptyOnly") {
					let n = e.pathFromRoot,
						o = 0;
					if ("always" !== t)
						for (o = n.length - 1; o >= 1; ) {
							let e = n[o],
								t = n[o - 1];
							if (e.routeConfig && "" === e.routeConfig.path) o--;
							else if (t.component) break;
							else o--;
						}
					return (function (e) {
						return e.reduce(
							(e, t) => {
								var n;
								let o = r._({}, e.params, t.params),
									i = r._({}, e.data, t.data),
									s = r._(
										{},
										t.data,
										e.resolve,
										null === (n = t.routeConfig) || void 0 === n
											? void 0
											: n.data,
										t._resolvedData,
									);
								return { params: o, data: i, resolve: s };
							},
							{ params: {}, data: {}, resolve: {} },
						);
					})(n.slice(o));
				}
				class ek {
					get title() {
						var e;
						return null === (e = this.data) || void 0 === e ? void 0 : e[c];
					}
					get root() {
						return this._routerState.root;
					}
					get parent() {
						return this._routerState.parent(this);
					}
					get firstChild() {
						return this._routerState.firstChild(this);
					}
					get children() {
						return this._routerState.children(this);
					}
					get pathFromRoot() {
						return this._routerState.pathFromRoot(this);
					}
					get paramMap() {
						return (
							!this._paramMap && (this._paramMap = h(this.params)),
							this._paramMap
						);
					}
					get queryParamMap() {
						return (
							!this._queryParamMap &&
								(this._queryParamMap = h(this.queryParams)),
							this._queryParamMap
						);
					}
					toString() {
						let e = this.url.map((e) => e.toString()).join("/"),
							t = this.routeConfig ? this.routeConfig.path : "";
						return `Route(url:'${e}', path:'${t}')`;
					}
					constructor(e, t, n, r, o, i, s, l, a) {
						(this.url = e),
							(this.params = t),
							(this.queryParams = n),
							(this.fragment = r),
							(this.data = o),
							(this.outlet = i),
							(this.component = s),
							(this.routeConfig = l),
							(this._resolve = a);
					}
				}
				class eF extends eM {
					toString() {
						return (function e(t) {
							let n =
								t.children.length > 0
									? ` { ${t.children.map(e).join(", ")} } `
									: "";
							return `${t.value}${n}`;
						})(this._root);
					}
					constructor(e, t) {
						super(t), (this.url = e), eR(this, t);
					}
				}
				function eR(e, t) {
					(t.value._routerState = e), t.children.forEach((t) => eR(e, t));
				}
				function eN(e) {
					if (e.snapshot) {
						let t = e.snapshot,
							n = e._futureSnapshot;
						(e.snapshot = n),
							!m(t.queryParams, n.queryParams) &&
								e.queryParamsSubject.next(n.queryParams),
							t.fragment !== n.fragment && e.fragmentSubject.next(n.fragment),
							!m(t.params, n.params) && e.paramsSubject.next(n.params),
							!(function (e, t) {
								if (e.length !== t.length) return !1;
								for (let n = 0; n < e.length; ++n)
									if (!m(e[n], t[n])) return !1;
								return !0;
							})(t.url, n.url) && e.urlSubject.next(n.url),
							!m(t.data, n.data) && e.dataSubject.next(n.data);
					} else
						(e.snapshot = e._futureSnapshot),
							e.dataSubject.next(e._futureSnapshot.data);
				}
				function eL(e, t) {
					var n, r;
					let o =
							m(e.params, t.params) &&
							((n = e.url),
							S(n, (r = t.url)) &&
								n.every((e, t) => m(e.parameters, r[t].parameters))),
						i = !e.parent != !t.parent;
					return o && !i && (!e.parent || eL(e.parent, t.parent));
				}
				let e$ = (() => {
					class e {
						get activatedComponentRef() {
							return this.activated;
						}
						ngOnChanges(e) {
							if (e.name) {
								let { firstChange: t, previousValue: n } = e.name;
								!t &&
									(this.isTrackedInParentContexts(n) &&
										(this.deactivate(),
										this.parentContexts.onChildOutletDestroyed(n)),
									this.initializeOutletWithName());
							}
						}
						ngOnDestroy() {
							var e;
							this.isTrackedInParentContexts(this.name) &&
								this.parentContexts.onChildOutletDestroyed(this.name),
								null === (e = this.inputBinder) ||
									void 0 === e ||
									e.unsubscribeFromRouteData(this);
						}
						isTrackedInParentContexts(e) {
							var t;
							return (
								(null === (t = this.parentContexts.getContext(e)) ||
								void 0 === t
									? void 0
									: t.outlet) === this
							);
						}
						ngOnInit() {
							this.initializeOutletWithName();
						}
						initializeOutletWithName() {
							if (
								(this.parentContexts.onChildOutletCreated(this.name, this),
								this.activated)
							)
								return;
							let e = this.parentContexts.getContext(this.name);
							(null == e ? void 0 : e.route) &&
								(e.attachRef
									? this.attach(e.attachRef, e.route)
									: this.activateWith(e.route, e.injector));
						}
						get isActivated() {
							return !!this.activated;
						}
						get component() {
							if (!this.activated)
								throw new i.ɵRuntimeError(
									4012,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										"Outlet is not activated",
								);
							return this.activated.instance;
						}
						get activatedRoute() {
							if (!this.activated)
								throw new i.ɵRuntimeError(
									4012,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										"Outlet is not activated",
								);
							return this._activatedRoute;
						}
						get activatedRouteData() {
							return this._activatedRoute
								? this._activatedRoute.snapshot.data
								: {};
						}
						detach() {
							if (!this.activated)
								throw new i.ɵRuntimeError(
									4012,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										"Outlet is not activated",
								);
							this.location.detach();
							let e = this.activated;
							return (
								(this.activated = null),
								(this._activatedRoute = null),
								this.detachEvents.emit(e.instance),
								e
							);
						}
						attach(e, t) {
							var n;
							(this.activated = e),
								(this._activatedRoute = t),
								this.location.insert(e.hostView),
								null === (n = this.inputBinder) ||
									void 0 === n ||
									n.bindActivatedRouteToOutletComponent(this),
								this.attachEvents.emit(e.instance);
						}
						deactivate() {
							if (this.activated) {
								let e = this.component;
								this.activated.destroy(),
									(this.activated = null),
									(this._activatedRoute = null),
									this.deactivateEvents.emit(e);
							}
						}
						activateWith(e, t) {
							var n;
							if (this.isActivated)
								throw new i.ɵRuntimeError(
									4013,
									("undefined" == typeof ngDevMode || ngDevMode) &&
										"Cannot activate an already activated outlet",
								);
							this._activatedRoute = e;
							let r = this.location,
								o = e.snapshot,
								s = o.component,
								l = this.parentContexts.getOrCreateContext(this.name).children,
								a = new eV(e, l, r.injector);
							(this.activated = r.createComponent(s, {
								index: r.length,
								injector: a,
								environmentInjector: null != t ? t : this.environmentInjector,
							})),
								this.changeDetector.markForCheck(),
								null === (n = this.inputBinder) ||
									void 0 === n ||
									n.bindActivatedRouteToOutletComponent(this),
								this.activateEvents.emit(this.activated.instance);
						}
						constructor() {
							(this.activated = null),
								(this._activatedRoute = null),
								(this.name = d),
								(this.activateEvents = new i.EventEmitter()),
								(this.deactivateEvents = new i.EventEmitter()),
								(this.attachEvents = new i.EventEmitter()),
								(this.detachEvents = new i.EventEmitter()),
								(this.parentContexts = (0, i.inject)(ew)),
								(this.location = (0, i.inject)(i.ViewContainerRef)),
								(this.changeDetector = (0, i.inject)(i.ChangeDetectorRef)),
								(this.environmentInjector = (0, i.inject)(
									i.EnvironmentInjector,
								)),
								(this.inputBinder = (0, i.inject)(eB, { optional: !0 })),
								(this.supportsBindingToComponentInputs = !0);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["router-outlet"]],
							inputs: { name: "name" },
							outputs: {
								activateEvents: "activate",
								deactivateEvents: "deactivate",
								attachEvents: "attach",
								detachEvents: "detach",
							},
							exportAs: ["outlet"],
							standalone: !0,
							features: [i.ɵɵNgOnChangesFeature],
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				class eV {
					get(e, t) {
						return e === eP
							? this.route
							: e === ew
							? this.childContexts
							: this.parent.get(e, t);
					}
					constructor(e, t, n) {
						(this.route = e), (this.childContexts = t), (this.parent = n);
					}
				}
				let eB = new i.InjectionToken(""),
					eU = (() => {
						class e {
							bindActivatedRouteToOutletComponent(e) {
								this.unsubscribeFromRouteData(e), this.subscribeToRouteData(e);
							}
							unsubscribeFromRouteData(e) {
								var t;
								null === (t = this.outletDataSubscriptions.get(e)) ||
									void 0 === t ||
									t.unsubscribe(),
									this.outletDataSubscriptions.delete(e);
							}
							subscribeToRouteData(e) {
								let { activatedRoute: t } = e,
									n = (0, s.combineLatest)([t.queryParams, t.params, t.data])
										.pipe(
											(0, a.switchMap)(([e, t, n], o) =>
												((n = r._({}, e, t, n)), 0 === o)
													? (0, s.of)(n)
													: Promise.resolve(n),
											),
										)
										.subscribe((n) => {
											if (
												!e.isActivated ||
												!e.activatedComponentRef ||
												e.activatedRoute !== t ||
												null === t.component
											) {
												this.unsubscribeFromRouteData(e);
												return;
											}
											let r = (0, i.reflectComponentType)(t.component);
											if (!r) {
												this.unsubscribeFromRouteData(e);
												return;
											}
											for (let { templateName: t } of r.inputs)
												e.activatedComponentRef.setInput(t, n[t]);
										});
								this.outletDataSubscriptions.set(e, n);
							}
							constructor() {
								this.outletDataSubscriptions = new Map();
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function eH(e, t, n) {
					if (n && e.shouldReuseRoute(t.value, n.value.snapshot)) {
						let r = n.value;
						r._futureSnapshot = t.value;
						let o = (function (e, t, n) {
							return t.children.map((t) => {
								for (let r of n.children)
									if (e.shouldReuseRoute(t.value, r.value.snapshot))
										return eH(e, t, r);
								return eH(e, t);
							});
						})(e, t, n);
						return new eE(r, o);
					}
					{
						if (e.shouldAttach(t.value)) {
							let n = e.retrieve(t.value);
							if (null !== n) {
								let r = n.route;
								return (
									(r.value._futureSnapshot = t.value),
									(r.children = t.children.map((t) => eH(e, t))),
									r
								);
							}
						}
						let n = (function (e) {
								return new eP(
									new s.BehaviorSubject(e.url),
									new s.BehaviorSubject(e.params),
									new s.BehaviorSubject(e.queryParams),
									new s.BehaviorSubject(e.fragment),
									new s.BehaviorSubject(e.data),
									e.outlet,
									e.component,
									e,
								);
							})(t.value),
							r = t.children.map((t) => eH(e, t));
						return new eE(n, r);
					}
				}
				let ez = "ngNavigationCancelingError";
				function eW(e, t) {
					let { redirectTo: n, navigationBehaviorOptions: r } = W(t)
							? { redirectTo: t, navigationBehaviorOptions: void 0 }
							: t,
						o = eq(ngDevMode && `Redirecting to "${e.serialize(n)}"`, 0, t);
					return (o.url = n), (o.navigationBehaviorOptions = r), o;
				}
				function eq(e, t, n) {
					let r = Error("NavigationCancelingError: " + (e || ""));
					return (r[ez] = !0), (r.cancellationCode = t), n && (r.url = n), r;
				}
				function eG(e) {
					return (
						(function (e) {
							return e && e[ez];
						})(e) && W(e.url)
					);
				}
				function eZ(e) {
					return e && e[ez];
				}
				let eY = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵcmp = i.ɵɵdefineComponent({
							type: e,
							selectors: [["ng-component"]],
							standalone: !0,
							features: [i.ɵɵStandaloneFeature],
							decls: 1,
							vars: 0,
							template: function (e, t) {
								1 & e && i.ɵɵelement(0, "router-outlet");
							},
							dependencies: [e$],
							encapsulation: 2,
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function eQ(e, t = "", n = !1) {
					for (let r = 0; r < e.length; r++) {
						let o = e[r],
							s = (function (e, t) {
								if (!t) return e;
								if (!e && !t.path) return "";
								if (e && !t.path) return `${e}/`;
								if (!e && t.path) return t.path;
								else return `${e}/${t.path}`;
							})(t, o);
						(function (e, t, n) {
							if ("undefined" == typeof ngDevMode || ngDevMode) {
								if (!e)
									throw new i.ɵRuntimeError(
										4014,
										`
      Invalid configuration of route '${t}': Encountered undefined route.
      The reason might be an extra comma.

      Example:
      const routes: Routes = [
        { path: '', redirectTo: '/dashboard', pathMatch: 'full' },
        { path: 'dashboard',  component: DashboardComponent },, << two commas
        { path: 'detail/:id', component: HeroDetailComponent }
      ];
    `,
									);
								if (Array.isArray(e))
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': Array cannot be specified`,
									);
								if (
									!e.redirectTo &&
									!e.component &&
									!e.loadComponent &&
									!e.children &&
									!e.loadChildren &&
									e.outlet &&
									e.outlet !== d
								)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': a componentless route without children or loadChildren cannot have a named outlet set`,
									);
								if (e.redirectTo && e.children)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': redirectTo and children cannot be used together`,
									);
								if (e.redirectTo && e.loadChildren)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': redirectTo and loadChildren cannot be used together`,
									);
								if (e.children && e.loadChildren)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': children and loadChildren cannot be used together`,
									);
								if (e.redirectTo && (e.component || e.loadComponent))
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': redirectTo and component/loadComponent cannot be used together`,
									);
								if (e.component && e.loadComponent)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': component and loadComponent cannot be used together`,
									);
								if (e.redirectTo && e.canActivate)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': redirectTo and canActivate cannot be used together. Redirects happen before activation so canActivate will never be executed.`,
									);
								if (e.path && e.matcher)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': path and matcher cannot be used together`,
									);
								if (
									void 0 === e.redirectTo &&
									!e.component &&
									!e.loadComponent &&
									!e.children &&
									!e.loadChildren
								)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}'. One of the following must be provided: component, loadComponent, redirectTo, children or loadChildren`,
									);
								if (void 0 === e.path && void 0 === e.matcher)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': routes must have either a path or a matcher specified`,
									);
								if ("string" == typeof e.path && "/" === e.path.charAt(0))
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '${t}': path cannot start with a slash`,
									);
								if (
									"" === e.path &&
									void 0 !== e.redirectTo &&
									void 0 === e.pathMatch
								)
									throw new i.ɵRuntimeError(
										4014,
										`Invalid configuration of route '{path: "${t}", redirectTo: "${e.redirectTo}"}': please provide 'pathMatch'. The default value of 'pathMatch' is 'prefix', but often the intent is to use 'full'.`,
									);
								n && eK(t, e.component);
							}
							e.children && eQ(e.children, t, n);
						})(o, s, n);
					}
				}
				function eK(e, t) {
					if (t && (0, i.ɵisNgModule)(t))
						throw new i.ɵRuntimeError(
							4014,
							`Invalid configuration of route '${e}'. You are using 'loadComponent' with a module, but it must be used with standalone components. Use 'loadChildren' instead.`,
						);
					if (t && !(0, i.isStandalone)(t))
						throw new i.ɵRuntimeError(
							4014,
							`Invalid configuration of route '${e}'. The component must be standalone.`,
						);
				}
				function eJ(e) {
					let t = e.children && e.children.map(eJ),
						n = t ? o._(r._({}, e), { children: t }) : r._({}, e);
					return (
						!n.component &&
							!n.loadComponent &&
							(t || n.loadChildren) &&
							n.outlet &&
							n.outlet !== d &&
							(n.component = eY),
						n
					);
				}
				function eX(e) {
					return e.outlet || d;
				}
				function e0(e) {
					var t;
					if (!e) return null;
					if (
						null === (t = e.routeConfig) || void 0 === t ? void 0 : t._injector
					)
						return e.routeConfig._injector;
					for (let t = e.parent; t; t = t.parent) {
						let e = t.routeConfig;
						if (null == e ? void 0 : e._loadedInjector)
							return e._loadedInjector;
						if (null == e ? void 0 : e._injector) return e._injector;
					}
					return null;
				}
				let e1 = !1,
					e5 = (e, t, n, r) =>
						(0, a.map)(
							(o) => (
								new e2(
									t,
									o.targetRouterState,
									o.currentRouterState,
									n,
									r,
								).activate(e),
								o
							),
						);
				class e2 {
					activate(e) {
						let t = this.futureState._root,
							n = this.currState ? this.currState._root : null;
						this.deactivateChildRoutes(t, n, e),
							eN(this.futureState.root),
							this.activateChildRoutes(t, n, e);
					}
					deactivateChildRoutes(e, t, n) {
						let r = eO(t);
						e.children.forEach((e) => {
							let t = e.value.outlet;
							this.deactivateRoutes(e, r[t], n), delete r[t];
						}),
							Object.values(r).forEach((e) => {
								this.deactivateRouteAndItsChildren(e, n);
							});
					}
					deactivateRoutes(e, t, n) {
						let r = e.value,
							o = t ? t.value : null;
						if (r === o) {
							if (r.component) {
								let o = n.getContext(r.outlet);
								o && this.deactivateChildRoutes(e, t, o.children);
							} else this.deactivateChildRoutes(e, t, n);
						} else o && this.deactivateRouteAndItsChildren(t, n);
					}
					deactivateRouteAndItsChildren(e, t) {
						e.value.component &&
						this.routeReuseStrategy.shouldDetach(e.value.snapshot)
							? this.detachAndStoreRouteSubtree(e, t)
							: this.deactivateRouteAndOutlet(e, t);
					}
					detachAndStoreRouteSubtree(e, t) {
						let n = t.getContext(e.value.outlet),
							r = n && e.value.component ? n.children : t,
							o = eO(e);
						for (let e of Object.keys(o))
							this.deactivateRouteAndItsChildren(o[e], r);
						if (n && n.outlet) {
							let t = n.outlet.detach(),
								r = n.children.onOutletDeactivated();
							this.routeReuseStrategy.store(e.value.snapshot, {
								componentRef: t,
								route: e,
								contexts: r,
							});
						}
					}
					deactivateRouteAndOutlet(e, t) {
						let n = t.getContext(e.value.outlet),
							r = n && e.value.component ? n.children : t,
							o = eO(e);
						for (let e of Object.keys(o))
							this.deactivateRouteAndItsChildren(o[e], r);
						n &&
							(n.outlet &&
								(n.outlet.deactivate(), n.children.onOutletDeactivated()),
							(n.attachRef = null),
							(n.route = null));
					}
					activateChildRoutes(e, t, n) {
						let r = eO(t);
						e.children.forEach((e) => {
							this.activateRoutes(e, r[e.value.outlet], n),
								this.forwardEvent(new ej(e.value.snapshot));
						}),
							e.children.length && this.forwardEvent(new eb(e.value.snapshot));
					}
					activateRoutes(e, t, n) {
						let r = e.value,
							o = t ? t.value : null;
						if ((eN(r), r === o)) {
							if (r.component) {
								let o = n.getOrCreateContext(r.outlet);
								this.activateChildRoutes(e, t, o.children);
							} else this.activateChildRoutes(e, t, n);
						} else if (r.component) {
							let t = n.getOrCreateContext(r.outlet);
							if (this.routeReuseStrategy.shouldAttach(r.snapshot)) {
								let n = this.routeReuseStrategy.retrieve(r.snapshot);
								this.routeReuseStrategy.store(r.snapshot, null),
									t.children.onOutletReAttached(n.contexts),
									(t.attachRef = n.componentRef),
									(t.route = n.route.value),
									t.outlet && t.outlet.attach(n.componentRef, n.route.value),
									eN(n.route.value),
									this.activateChildRoutes(e, null, t.children);
							} else {
								let n = e0(r.snapshot);
								(t.attachRef = null),
									(t.route = r),
									(t.injector = n),
									t.outlet && t.outlet.activateWith(r, t.injector),
									this.activateChildRoutes(e, null, t.children);
							}
						} else this.activateChildRoutes(e, null, n);
						if ("undefined" == typeof ngDevMode || ngDevMode) {
							let e = n.getOrCreateContext(r.outlet),
								t = e.outlet;
							t &&
								this.inputBindingEnabled &&
								!t.supportsBindingToComponentInputs &&
								!e1 &&
								(console.warn(
									"'withComponentInputBinding' feature is enabled but this application is using an outlet that may not support binding to component inputs.",
								),
								(e1 = !0));
						}
					}
					constructor(e, t, n, r, o) {
						(this.routeReuseStrategy = e),
							(this.futureState = t),
							(this.currState = n),
							(this.forwardEvent = r),
							(this.inputBindingEnabled = o);
					}
				}
				class e3 {
					constructor(e) {
						(this.path = e), (this.route = this.path[this.path.length - 1]);
					}
				}
				class e4 {
					constructor(e, t) {
						(this.component = e), (this.route = t);
					}
				}
				function e6(e, t) {
					let n = Symbol(),
						r = t.get(e, n);
					if (r === n)
						return "function" != typeof e || (0, i.ɵisInjectable)(e)
							? t.get(e)
							: e;
					return r;
				}
				function e8(
					e,
					t,
					n,
					r,
					o = { canDeactivateChecks: [], canActivateChecks: [] },
				) {
					let i = eO(t);
					return (
						e.children.forEach((e) => {
							(function (
								e,
								t,
								n,
								r,
								o = { canDeactivateChecks: [], canActivateChecks: [] },
							) {
								let i = e.value,
									s = t ? t.value : null,
									l = n ? n.getContext(e.value.outlet) : null;
								if (s && i.routeConfig === s.routeConfig) {
									let a = (function (e, t, n) {
										if ("function" == typeof n) return n(e, t);
										switch (n) {
											case "pathParamsChange":
												return !S(e.url, t.url);
											case "pathParamsOrQueryParamsChange":
												return (
													!S(e.url, t.url) || !m(e.queryParams, t.queryParams)
												);
											case "always":
												return !0;
											case "paramsOrQueryParamsChange":
												return !eL(e, t) || !m(e.queryParams, t.queryParams);
											default:
												return !eL(e, t);
										}
									})(s, i, i.routeConfig.runGuardsAndResolvers);
									a
										? o.canActivateChecks.push(new e3(r))
										: ((i.data = s.data), (i._resolvedData = s._resolvedData)),
										i.component
											? e8(e, t, l ? l.children : null, r, o)
											: e8(e, t, n, r, o),
										a &&
											l &&
											l.outlet &&
											l.outlet.isActivated &&
											o.canDeactivateChecks.push(new e4(l.outlet.component, s));
								} else
									s && e7(t, l, o),
										o.canActivateChecks.push(new e3(r)),
										i.component
											? e8(e, null, l ? l.children : null, r, o)
											: e8(e, null, n, r, o);
							})(e, i[e.value.outlet], n, r.concat([e.value]), o),
								delete i[e.value.outlet];
						}),
						Object.entries(i).forEach(([e, t]) => e7(t, n.getContext(e), o)),
						o
					);
				}
				function e7(e, t, n) {
					let r = eO(e),
						o = e.value;
					Object.entries(r).forEach(([e, r]) => {
						o.component
							? t
								? e7(r, t.children.getContext(e), n)
								: e7(r, null, n)
							: e7(r, t, n);
					}),
						o.component && t && t.outlet && t.outlet.isActivated
							? n.canDeactivateChecks.push(new e4(t.outlet.component, o))
							: n.canDeactivateChecks.push(new e4(null, o));
				}
				function e9(e) {
					return "function" == typeof e;
				}
				function te(e) {
					return (
						e instanceof s.EmptyError ||
						(null == e ? void 0 : e.name) === "EmptyError"
					);
				}
				let tt = Symbol("INITIAL_VALUE");
				function tn() {
					return (0, a.switchMap)((e) =>
						(0, s.combineLatest)(
							e.map((e) => e.pipe((0, a.take)(1), (0, a.startWith)(tt))),
						).pipe(
							(0, a.map)((e) => {
								for (let t of e) {
									if (!0 !== t) {
										if (t === tt) return tt;
										else if (!1 === t || t instanceof w) return t;
									}
								}
								return !0;
							}),
							(0, a.filter)((e) => e !== tt),
							(0, a.take)(1),
						),
					);
				}
				function tr(e) {
					return (0, s.pipe)(
						(0, a.tap)((t) => {
							if (W(t)) throw eW(e, t);
						}),
						(0, a.map)((e) => !0 === e),
					);
				}
				class to {
					constructor(e) {
						this.segmentGroup = e || null;
					}
				}
				class ti {
					constructor(e) {
						this.urlTree = e;
					}
				}
				function ts(e) {
					return (0, s.throwError)(new to(e));
				}
				function tl(e) {
					return (0, s.throwError)(new ti(e));
				}
				class ta {
					noMatchError(e) {
						return new i.ɵRuntimeError(
							4002,
							("undefined" == typeof ngDevMode || ngDevMode) &&
								`Cannot match any routes. URL Segment: '${e.segmentGroup}'`,
						);
					}
					lineralizeSegments(e, t) {
						let n = [],
							r = t.root;
						for (;;) {
							if (((n = n.concat(r.segments)), 0 === r.numberOfChildren))
								return (0, s.of)(n);
							if (r.numberOfChildren > 1 || !r.children[d]) {
								var o;
								return (
									(o = e.redirectTo),
									(0, s.throwError)(
										new i.ɵRuntimeError(
											4e3,
											("undefined" == typeof ngDevMode || ngDevMode) &&
												`Only absolute redirects can have named outlets. redirectTo: '${o}'`,
										),
									)
								);
							}
							r = r.children[d];
						}
					}
					applyRedirectCommands(e, t, n) {
						return this.applyRedirectCreateUrlTree(
							t,
							this.urlSerializer.parse(t),
							e,
							n,
						);
					}
					applyRedirectCreateUrlTree(e, t, n, r) {
						let o = this.createSegmentGroup(e, t.root, n, r);
						return new w(
							o,
							this.createQueryParams(t.queryParams, this.urlTree.queryParams),
							t.fragment,
						);
					}
					createQueryParams(e, t) {
						let n = {};
						return (
							Object.entries(e).forEach(([e, r]) => {
								let o = "string" == typeof r && r.startsWith(":");
								if (o) {
									let o = r.substring(1);
									n[e] = t[o];
								} else n[e] = r;
							}),
							n
						);
					}
					createSegmentGroup(e, t, n, r) {
						let o = this.createSegments(e, t.segments, n, r),
							i = {};
						return (
							Object.entries(t.children).forEach(([t, o]) => {
								i[t] = this.createSegmentGroup(e, o, n, r);
							}),
							new M(o, i)
						);
					}
					createSegments(e, t, n, r) {
						return t.map((t) =>
							t.path.startsWith(":")
								? this.findPosParam(e, t, r)
								: this.findOrReturn(t, n),
						);
					}
					findPosParam(e, t, n) {
						let r = n[t.path.substring(1)];
						if (!r)
							throw new i.ɵRuntimeError(
								4001,
								("undefined" == typeof ngDevMode || ngDevMode) &&
									`Cannot redirect to '${e}'. Cannot find '${t.path}'.`,
							);
						return r;
					}
					findOrReturn(e, t) {
						let n = 0;
						for (let r of t) {
							if (r.path === e.path) return t.splice(n), r;
							n++;
						}
						return e;
					}
					constructor(e, t) {
						(this.urlSerializer = e), (this.urlTree = t);
					}
				}
				let tu = {
					matched: !1,
					consumedSegments: [],
					remainingSegments: [],
					parameters: {},
					positionalParamSegments: {},
				};
				function td(e, t, n) {
					var o, i;
					if ("" === t.path)
						return "full" === t.pathMatch && (e.hasChildren() || n.length > 0)
							? r._({}, tu)
							: {
									matched: !0,
									consumedSegments: [],
									remainingSegments: n,
									parameters: {},
									positionalParamSegments: {},
							  };
					let s = t.matcher || p,
						l = s(n, e, t);
					if (!l) return r._({}, tu);
					let a = {};
					Object.entries(
						null !== (o = l.posParams) && void 0 !== o ? o : {},
					).forEach(([e, t]) => {
						a[e] = t.path;
					});
					let u =
						l.consumed.length > 0
							? r._({}, a, l.consumed[l.consumed.length - 1].parameters)
							: a;
					return {
						matched: !0,
						consumedSegments: l.consumed,
						remainingSegments: n.slice(l.consumed.length),
						parameters: u,
						positionalParamSegments:
							null !== (i = l.posParams) && void 0 !== i ? i : {},
					};
				}
				function tc(e, t, n, o) {
					if (
						n.length > 0 &&
						(function (e, t, n) {
							return n.some((n) => tf(e, t, n) && eX(n) !== d);
						})(e, n, o)
					) {
						let r = new M(
							t,
							(function (e, t) {
								let n = {};
								for (let r of ((n[d] = t), e))
									if ("" === r.path && eX(r) !== d) {
										let e = new M([], {});
										n[eX(r)] = e;
									}
								return n;
							})(o, new M(n, e.children)),
						);
						return { segmentGroup: r, slicedSegments: [] };
					}
					if (
						0 === n.length &&
						(function (e, t, n) {
							return n.some((n) => tf(e, t, n));
						})(e, n, o)
					) {
						let i = new M(
							e.segments,
							(function (e, t, n, o, i) {
								let s = {};
								for (let t of o)
									if (tf(e, n, t) && !i[eX(t)]) {
										let e = new M([], {});
										s[eX(t)] = e;
									}
								return r._({}, i, s);
							})(e, t, n, o, e.children),
						);
						return { segmentGroup: i, slicedSegments: n };
					}
					let i = new M(e.segments, e.children);
					return { segmentGroup: i, slicedSegments: n };
				}
				function tf(e, t, n) {
					return (
						((!e.hasChildren() && !(t.length > 0)) || "full" !== n.pathMatch) &&
						"" === n.path
					);
				}
				class th {
					noMatchError(e) {
						return new i.ɵRuntimeError(
							4002,
							("undefined" == typeof ngDevMode || ngDevMode) &&
								`Cannot match any routes. URL Segment: '${e.segmentGroup}'`,
						);
					}
					recognize() {
						let e = tc(this.urlTree.root, [], [], this.config).segmentGroup;
						return this.processSegmentGroup(
							this.injector,
							this.config,
							e,
							d,
						).pipe(
							(0, a.catchError)((e) => {
								if (e instanceof ti)
									return (
										(this.allowRedirects = !1),
										(this.urlTree = e.urlTree),
										this.match(e.urlTree)
									);
								if (e instanceof to) throw this.noMatchError(e);
								throw e;
							}),
							(0, a.map)((e) => {
								let t = new ek(
										[],
										Object.freeze({}),
										Object.freeze(r._({}, this.urlTree.queryParams)),
										this.urlTree.fragment,
										{},
										d,
										this.rootComponentType,
										null,
										{},
									),
									n = new eE(t, e),
									o = new eF("", n),
									i = (function (e, t, n = null, r = null) {
										let o = q(e);
										return G(o, t, n, r);
									})(t, [], this.urlTree.queryParams, this.urlTree.fragment);
								return (
									(i.queryParams = this.urlTree.queryParams),
									(o.url = this.urlSerializer.serialize(i)),
									this.inheritParamsAndData(o._root),
									{ state: o, tree: i }
								);
							}),
						);
					}
					match(e) {
						let t = this.processSegmentGroup(
							this.injector,
							this.config,
							e.root,
							d,
						);
						return t.pipe(
							(0, a.catchError)((e) => {
								if (e instanceof to) throw this.noMatchError(e);
								throw e;
							}),
						);
					}
					inheritParamsAndData(e) {
						let t = e.value,
							n = eT(t, this.paramsInheritanceStrategy);
						(t.params = Object.freeze(n.params)),
							(t.data = Object.freeze(n.data)),
							e.children.forEach((e) => this.inheritParamsAndData(e));
					}
					processSegmentGroup(e, t, n, r) {
						return 0 === n.segments.length && n.hasChildren()
							? this.processChildren(e, t, n)
							: this.processSegment(e, t, n, n.segments, r, !0);
					}
					processChildren(e, t, n) {
						let r = [];
						for (let e of Object.keys(n.children))
							"primary" === e ? r.unshift(e) : r.push(e);
						return (0, s.from)(r).pipe(
							(0, a.concatMap)((r) => {
								let o = n.children[r],
									i = (function (e, t) {
										let n = e.filter((e) => eX(e) === t);
										return n.push(...e.filter((e) => eX(e) !== t)), n;
									})(t, r);
								return this.processSegmentGroup(e, i, o, r);
							}),
							(0, a.scan)((e, t) => (e.push(...t), e)),
							(0, a.defaultIfEmpty)(null),
							(0, a.last)(),
							(0, a.mergeMap)((e) => {
								if (null === e) return ts(n);
								let t = (function e(t) {
									let n = [],
										r = new Set();
									for (let e of t) {
										if (
											!(function (e) {
												let t = e.value.routeConfig;
												return t && "" === t.path;
											})(e)
										) {
											n.push(e);
											continue;
										}
										let t = n.find(
											(t) => e.value.routeConfig === t.value.routeConfig,
										);
										void 0 !== t
											? (t.children.push(...e.children), r.add(t))
											: n.push(e);
									}
									for (let t of r) {
										let r = e(t.children);
										n.push(new eE(t.value, r));
									}
									return n.filter((e) => !r.has(e));
								})(e);
								return (
									("undefined" == typeof ngDevMode || ngDevMode) &&
										(function (e) {
											let t = {};
											e.forEach((e) => {
												let n = t[e.value.outlet];
												if (n) {
													let t = n.url.map((e) => e.toString()).join("/"),
														r = e.value.url.map((e) => e.toString()).join("/");
													throw new i.ɵRuntimeError(
														4006,
														("undefined" == typeof ngDevMode || ngDevMode) &&
															`Two segments cannot have the same outlet name: '${t}' and '${r}'.`,
													);
												}
												t[e.value.outlet] = e.value;
											});
										})(t),
									(function (e) {
										e.sort((e, t) =>
											e.value.outlet === d
												? -1
												: t.value.outlet === d
												? 1
												: e.value.outlet.localeCompare(t.value.outlet),
										);
									})(t),
									(0, s.of)(t)
								);
							}),
						);
					}
					processSegment(e, t, n, r, o, i) {
						return (0, s.from)(t).pipe(
							(0, a.concatMap)((l) => {
								var u;
								return this.processSegmentAgainstRoute(
									null !== (u = l._injector) && void 0 !== u ? u : e,
									t,
									l,
									n,
									r,
									o,
									i,
								).pipe(
									(0, a.catchError)((e) => {
										if (e instanceof to) return (0, s.of)(null);
										throw e;
									}),
								);
							}),
							(0, a.first)((e) => !!e),
							(0, a.catchError)((e) => {
								if (te(e)) {
									var t, i, l;
									return ((t = n),
									(i = r),
									(l = o),
									0 !== i.length || t.children[l])
										? ts(n)
										: (0, s.of)([]);
								}
								throw e;
							}),
						);
					}
					processSegmentAgainstRoute(e, t, n, r, o, i, s) {
						var l, a, u, c;
						return ((l = n),
						(a = r),
						(u = o),
						(c = i),
						(eX(l) === c || (c !== d && tf(a, u, l))) &&
							("**" === l.path || td(a, l, u).matched))
							? void 0 === n.redirectTo
								? this.matchSegmentAgainstRoute(e, r, n, o, i, s)
								: s && this.allowRedirects
								? this.expandSegmentAgainstRouteUsingRedirect(e, r, t, n, o, i)
								: ts(r)
							: ts(r);
					}
					expandSegmentAgainstRouteUsingRedirect(e, t, n, r, o, i) {
						return "**" === r.path
							? this.expandWildCardWithParamsAgainstRouteUsingRedirect(
									e,
									n,
									r,
									i,
							  )
							: this.expandRegularSegmentAgainstRouteUsingRedirect(
									e,
									t,
									n,
									r,
									o,
									i,
							  );
					}
					expandWildCardWithParamsAgainstRouteUsingRedirect(e, t, n, r) {
						let o = this.applyRedirects.applyRedirectCommands(
							[],
							n.redirectTo,
							{},
						);
						return n.redirectTo.startsWith("/")
							? tl(o)
							: this.applyRedirects.lineralizeSegments(n, o).pipe(
									(0, a.mergeMap)((n) => {
										let o = new M(n, {});
										return this.processSegment(e, t, o, n, r, !1);
									}),
							  );
					}
					expandRegularSegmentAgainstRouteUsingRedirect(e, t, n, r, o, i) {
						let {
							matched: s,
							consumedSegments: l,
							remainingSegments: u,
							positionalParamSegments: d,
						} = td(t, r, o);
						if (!s) return ts(t);
						let c = this.applyRedirects.applyRedirectCommands(
							l,
							r.redirectTo,
							d,
						);
						return r.redirectTo.startsWith("/")
							? tl(c)
							: this.applyRedirects
									.lineralizeSegments(r, c)
									.pipe(
										(0, a.mergeMap)((r) =>
											this.processSegment(e, n, t, r.concat(u), i, !1),
										),
									);
					}
					matchSegmentAgainstRoute(e, t, n, o, l, u) {
						let c;
						if ("**" === n.path) {
							var f, h;
							let e = o.length > 0 ? v(o).parameters : {},
								i = new ek(
									o,
									e,
									Object.freeze(r._({}, this.urlTree.queryParams)),
									this.urlTree.fragment,
									(function (e) {
										return e.data || {};
									})(n),
									eX(n),
									null !==
										(h =
											null !== (f = n.component) && void 0 !== f
												? f
												: n._loadedComponent) && void 0 !== h
										? h
										: null,
									n,
									tm(n),
								);
							(c = (0, s.of)({
								snapshot: i,
								consumedSegments: [],
								remainingSegments: [],
							})),
								(t.children = {});
						} else
							c = (function (e, t, n, o, l) {
								var u, d, c;
								let f = td(e, t, n);
								if (!f.matched) return (0, s.of)(f);
								return (
									(u = t),
									(d = o),
									u.providers &&
										!u._injector &&
										(u._injector = (0, i.createEnvironmentInjector)(
											u.providers,
											d,
											`Route: ${u.path}`,
										)),
									(function (e, t, n, r) {
										let o = t.canMatch;
										if (!o || 0 === o.length) return (0, s.of)(!0);
										let i = o.map((r) => {
											var o;
											let i = e6(r, e);
											let s =
												(o = i) && e9(o.canMatch)
													? i.canMatch(t, n)
													: e.runInContext(() => i(t, n));
											return y(s);
										});
										return (0, s.of)(i).pipe(tn(), tr(r));
									})(
										(o = null !== (c = u._injector) && void 0 !== c ? c : d),
										t,
										n,
										l,
									).pipe((0, a.map)((e) => (!0 === e ? f : r._({}, tu))))
								);
							})(t, n, o, e, this.urlSerializer).pipe(
								(0, a.map)(
									({
										matched: e,
										consumedSegments: t,
										remainingSegments: o,
										parameters: i,
									}) => {
										var s, l;
										if (!e) return null;
										let a = new ek(
											t,
											i,
											Object.freeze(r._({}, this.urlTree.queryParams)),
											this.urlTree.fragment,
											(function (e) {
												return e.data || {};
											})(n),
											eX(n),
											null !==
												(l =
													null !== (s = n.component) && void 0 !== s
														? s
														: n._loadedComponent) && void 0 !== l
												? l
												: null,
											n,
											tm(n),
										);
										return {
											snapshot: a,
											consumedSegments: t,
											remainingSegments: o,
										};
									},
								),
							);
						return c.pipe(
							(0, a.switchMap)((r) => {
								var i;
								return null === r
									? ts(t)
									: ((e = null !== (i = n._injector) && void 0 !== i ? i : e),
									  this.getChildConfig(e, n, o).pipe(
											(0, a.switchMap)(({ routes: o }) => {
												var i;
												let u =
														null !== (i = n._loadedInjector) && void 0 !== i
															? i
															: e,
													{
														snapshot: c,
														consumedSegments: f,
														remainingSegments: h,
													} = r,
													{ segmentGroup: p, slicedSegments: m } = tc(
														t,
														f,
														h,
														o,
													);
												if (0 === m.length && p.hasChildren())
													return this.processChildren(u, o, p).pipe(
														(0, a.map)((e) =>
															null === e ? null : [new eE(c, e)],
														),
													);
												if (0 === o.length && 0 === m.length)
													return (0, s.of)([new eE(c, [])]);
												let g = eX(n) === l;
												return this.processSegment(
													u,
													o,
													p,
													m,
													g ? d : l,
													!0,
												).pipe((0, a.map)((e) => [new eE(c, e)]));
											}),
									  ));
							}),
						);
					}
					getChildConfig(e, t, n) {
						if (t.children)
							return (0, s.of)({ routes: t.children, injector: e });
						if (t.loadChildren)
							return void 0 !== t._loadedRoutes
								? (0, s.of)({
										routes: t._loadedRoutes,
										injector: t._loadedInjector,
								  })
								: (function (e, t, n, r) {
										let o = t.canLoad;
										if (void 0 === o || 0 === o.length) return (0, s.of)(!0);
										let i = o.map((r) => {
											var o;
											let i = e6(r, e);
											let s =
												(o = i) && e9(o.canLoad)
													? i.canLoad(t, n)
													: e.runInContext(() => i(t, n));
											return y(s);
										});
										return (0, s.of)(i).pipe(tn(), tr(r));
								  })(e, t, n, this.urlSerializer).pipe(
										(0, a.mergeMap)((n) => {
											var r;
											if (n)
												return this.configLoader.loadChildren(e, t).pipe(
													(0, a.tap)((e) => {
														(t._loadedRoutes = e.routes),
															(t._loadedInjector = e.injector);
													}),
												);
											return (
												(r = t),
												(0, s.throwError)(
													eq(
														("undefined" == typeof ngDevMode || ngDevMode) &&
															`Cannot load children because the guard of the route "path: '${r.path}'" returned false`,
														3,
													),
												)
											);
										}),
								  );
						return (0, s.of)({ routes: [], injector: e });
					}
					constructor(e, t, n, r, o, i, s) {
						(this.injector = e),
							(this.configLoader = t),
							(this.rootComponentType = n),
							(this.config = r),
							(this.urlTree = o),
							(this.paramsInheritanceStrategy = i),
							(this.urlSerializer = s),
							(this.allowRedirects = !0),
							(this.applyRedirects = new ta(this.urlSerializer, this.urlTree));
					}
				}
				function tp(e) {
					return e.data || {};
				}
				function tm(e) {
					return e.resolve || {};
				}
				function tg(e) {
					return "string" == typeof e.title || null === e.title;
				}
				function tv(e) {
					return (0, a.switchMap)((t) => {
						let n = e(t);
						return n ? (0, s.from)(n).pipe((0, a.map)(() => t)) : (0, s.of)(t);
					});
				}
				let ty = new i.InjectionToken("ROUTES"),
					tb = (() => {
						class e {
							loadComponent(e) {
								if (this.componentLoaders.get(e))
									return this.componentLoaders.get(e);
								if (e._loadedComponent) return (0, s.of)(e._loadedComponent);
								this.onLoadStartListener && this.onLoadStartListener(e);
								let t = y(e.loadComponent()).pipe(
										(0, a.map)(t_),
										(0, a.tap)((t) => {
											var n;
											this.onLoadEndListener && this.onLoadEndListener(e),
												("undefined" == typeof ngDevMode || ngDevMode) &&
													eK(null !== (n = e.path) && void 0 !== n ? n : "", t),
												(e._loadedComponent = t);
										}),
										(0, a.finalize)(() => {
											this.componentLoaders.delete(e);
										}),
									),
									n = new s.ConnectableObservable(
										t,
										() => new s.Subject(),
									).pipe((0, a.refCount)());
								return this.componentLoaders.set(e, n), n;
							}
							loadChildren(e, t) {
								if (this.childrenLoaders.get(t))
									return this.childrenLoaders.get(t);
								if (t._loadedRoutes)
									return (0, s.of)({
										routes: t._loadedRoutes,
										injector: t._loadedInjector,
									});
								this.onLoadStartListener && this.onLoadStartListener(t);
								let n = this.loadModuleFactoryOrRoutes(t.loadChildren),
									r = n.pipe(
										(0, a.map)((n) => {
											let r, o;
											this.onLoadEndListener && this.onLoadEndListener(t);
											let s = !1;
											Array.isArray(n)
												? ((o = n), (s = !0))
												: (o = (r = n.create(e).injector)
														.get(
															ty,
															[],
															i.InjectFlags.Self | i.InjectFlags.Optional,
														)
														.flat());
											let l = o.map(eJ);
											return (
												("undefined" == typeof ngDevMode || ngDevMode) &&
													eQ(l, t.path, s),
												{ routes: l, injector: r }
											);
										}),
										(0, a.finalize)(() => {
											this.childrenLoaders.delete(t);
										}),
									),
									o = new s.ConnectableObservable(
										r,
										() => new s.Subject(),
									).pipe((0, a.refCount)());
								return this.childrenLoaders.set(t, o), o;
							}
							loadModuleFactoryOrRoutes(e) {
								return y(e()).pipe(
									(0, a.map)(t_),
									(0, a.mergeMap)((e) =>
										e instanceof i.NgModuleFactory || Array.isArray(e)
											? (0, s.of)(e)
											: (0, s.from)(this.compiler.compileModuleAsync(e)),
									),
								);
							}
							constructor() {
								(this.componentLoaders = new WeakMap()),
									(this.childrenLoaders = new WeakMap()),
									(this.compiler = (0, i.inject)(i.Compiler));
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function t_(e) {
					var t;
					return (t = e) && "object" == typeof t && "default" in t
						? e.default
						: e;
				}
				let tj = (() => {
					class e {
						get hasRequestedNavigation() {
							return 0 !== this.navigationId;
						}
						complete() {
							var e;
							null === (e = this.transitions) || void 0 === e || e.complete();
						}
						handleNavigationRequest(e) {
							var t;
							let n = ++this.navigationId;
							null === (t = this.transitions) ||
								void 0 === t ||
								t.next(o._(r._({}, this.transitions.value, e), { id: n }));
						}
						setupNavigations(e) {
							return (
								(this.transitions = new s.BehaviorSubject({
									id: 0,
									currentUrlTree: e.currentUrlTree,
									currentRawUrl: e.currentUrlTree,
									extractedUrl: e.urlHandlingStrategy.extract(e.currentUrlTree),
									urlAfterRedirects: e.urlHandlingStrategy.extract(
										e.currentUrlTree,
									),
									rawUrl: e.currentUrlTree,
									extras: {},
									resolve: null,
									reject: null,
									promise: Promise.resolve(!0),
									source: eo,
									restoredState: null,
									currentSnapshot: e.routerState.snapshot,
									targetSnapshot: null,
									currentRouterState: e.routerState,
									targetRouterState: null,
									guards: { canActivateChecks: [], canDeactivateChecks: [] },
									guardsResult: null,
								})),
								this.transitions.pipe(
									(0, a.filter)((e) => 0 !== e.id),
									(0, a.map)((t) =>
										o._(r._({}, t), {
											extractedUrl: e.urlHandlingStrategy.extract(t.rawUrl),
										}),
									),
									(0, a.switchMap)((t) => {
										var n, i;
										let l = !1,
											u = !1;
										return (0, s.of)(t).pipe(
											(0, a.tap)((e) => {
												this.currentNavigation = {
													id: e.id,
													initialUrl: e.rawUrl,
													extractedUrl: e.extractedUrl,
													trigger: e.source,
													extras: e.extras,
													previousNavigation: this.lastSuccessfulNavigation
														? o._(r._({}, this.lastSuccessfulNavigation), {
																previousNavigation: null,
														  })
														: null,
												};
											}),
											(0, a.switchMap)((n) => {
												var i, l, u, d, c, f, h;
												let p = e.browserUrlTree.toString(),
													m =
														!e.navigated ||
														n.extractedUrl.toString() !== p ||
														p !== e.currentUrlTree.toString(),
													g =
														null !== (i = n.extras.onSameUrlNavigation) &&
														void 0 !== i
															? i
															: e.onSameUrlNavigation;
												if (!m && "reload" !== g) {
													let r =
														"undefined" == typeof ngDevMode || ngDevMode
															? `Navigation to ${n.rawUrl} was ignored because it is the same as the current Router URL.`
															: "";
													return (
														this.events.next(
															new eu(n.id, e.serializeUrl(t.rawUrl), r, 0),
														),
														(e.rawUrlTree = n.rawUrl),
														n.resolve(null),
														s.EMPTY
													);
												}
												if (e.urlHandlingStrategy.shouldProcessUrl(n.rawUrl)) {
													return (
														(function (e) {
															return e !== eo;
														})(n.source) && (e.browserUrlTree = n.extractedUrl),
														(0, s.of)(n).pipe(
															(0, a.switchMap)((e) => {
																var t, n;
																let r =
																	null === (t = this.transitions) ||
																	void 0 === t
																		? void 0
																		: t.getValue();
																return (this.events.next(
																	new es(
																		e.id,
																		this.urlSerializer.serialize(
																			e.extractedUrl,
																		),
																		e.source,
																		e.restoredState,
																	),
																),
																r !==
																	(null === (n = this.transitions) ||
																	void 0 === n
																		? void 0
																		: n.getValue()))
																	? s.EMPTY
																	: Promise.resolve(e);
															}),
															((l = this.environmentInjector),
															(u = this.configLoader),
															(d = this.rootComponentType),
															(c = e.config),
															(f = this.urlSerializer),
															(h = e.paramsInheritanceStrategy),
															(0, a.mergeMap)((e) =>
																(function (e, t, n, r, o, i, s = "emptyOnly") {
																	return new th(
																		e,
																		t,
																		n,
																		r,
																		o,
																		s,
																		i,
																	).recognize();
																})(l, u, d, c, e.extractedUrl, f, h).pipe(
																	(0, a.map)(({ state: t, tree: n }) =>
																		o._(r._({}, e), {
																			targetSnapshot: t,
																			urlAfterRedirects: n,
																		}),
																	),
																),
															)),
															(0, a.tap)((n) => {
																if (
																	((t.targetSnapshot = n.targetSnapshot),
																	(t.urlAfterRedirects = n.urlAfterRedirects),
																	(this.currentNavigation = o._(
																		r._({}, this.currentNavigation),
																		{ finalUrl: n.urlAfterRedirects },
																	)),
																	"eager" === e.urlUpdateStrategy)
																) {
																	if (!n.extras.skipLocationChange) {
																		let t = e.urlHandlingStrategy.merge(
																			n.urlAfterRedirects,
																			n.rawUrl,
																		);
																		e.setBrowserUrl(t, n);
																	}
																	e.browserUrlTree = n.urlAfterRedirects;
																}
																let i = new ec(
																	n.id,
																	this.urlSerializer.serialize(n.extractedUrl),
																	this.urlSerializer.serialize(
																		n.urlAfterRedirects,
																	),
																	n.targetSnapshot,
																);
																this.events.next(i);
															}),
														)
													);
												}
												if (
													m &&
													e.urlHandlingStrategy.shouldProcessUrl(e.rawUrlTree)
												) {
													let {
															id: e,
															extractedUrl: i,
															source: l,
															restoredState: a,
															extras: u,
														} = n,
														d = new es(
															e,
															this.urlSerializer.serialize(i),
															l,
															a,
														);
													this.events.next(d);
													let c = eI(i, this.rootComponentType).snapshot;
													return (
														(t = o._(r._({}, n), {
															targetSnapshot: c,
															urlAfterRedirects: i,
															extras: o._(r._({}, u), {
																skipLocationChange: !1,
																replaceUrl: !1,
															}),
														})),
														(0, s.of)(t)
													);
												}
												{
													let r =
														"undefined" == typeof ngDevMode || ngDevMode
															? `Navigation was ignored because the UrlHandlingStrategy indicated neither the current URL ${e.rawUrlTree} nor target URL ${n.rawUrl} should be processed.`
															: "";
													return (
														this.events.next(
															new eu(
																n.id,
																e.serializeUrl(t.extractedUrl),
																r,
																1,
															),
														),
														(e.rawUrlTree = n.rawUrl),
														n.resolve(null),
														s.EMPTY
													);
												}
											}),
											(0, a.tap)((e) => {
												let t = new ef(
													e.id,
													this.urlSerializer.serialize(e.extractedUrl),
													this.urlSerializer.serialize(e.urlAfterRedirects),
													e.targetSnapshot,
												);
												this.events.next(t);
											}),
											(0, a.map)(
												(e) =>
													(t = o._(r._({}, e), {
														guards: (function (e, t, n) {
															let r = e._root,
																o = t ? t._root : null;
															return e8(r, o, n, [r.value]);
														})(
															e.targetSnapshot,
															e.currentSnapshot,
															this.rootContexts,
														),
													})),
											),
											((n = this.environmentInjector),
											(i = (e) => this.events.next(e)),
											(0, a.mergeMap)((e) => {
												let {
													targetSnapshot: t,
													currentSnapshot: l,
													guards: {
														canActivateChecks: u,
														canDeactivateChecks: d,
													},
												} = e;
												return 0 === d.length && 0 === u.length
													? (0, s.of)(o._(r._({}, e), { guardsResult: !0 }))
													: (function (e, t, n, r) {
															return (0, s.from)(e).pipe(
																(0, a.mergeMap)((e) =>
																	(function (e, t, n, r, o) {
																		let i =
																			t && t.routeConfig
																				? t.routeConfig.canDeactivate
																				: null;
																		if (!i || 0 === i.length)
																			return (0, s.of)(!0);
																		let l = i.map((i) => {
																			var s, l;
																			let u =
																					null !== (s = e0(t)) && void 0 !== s
																						? s
																						: o,
																				d = e6(i, u);
																			let c =
																				(l = d) && e9(l.canDeactivate)
																					? d.canDeactivate(e, t, n, r)
																					: u.runInContext(() => d(e, t, n, r));
																			return y(c).pipe((0, a.first)());
																		});
																		return (0, s.of)(l).pipe(tn());
																	})(e.component, e.route, n, t, r),
																),
																(0, a.first)((e) => !0 !== e, !0),
															);
													  })(d, t, l, n).pipe(
															(0, a.mergeMap)((e) =>
																e && "boolean" == typeof e
																	? (function (e, t, n, r) {
																			return (0, s.from)(t).pipe(
																				(0, a.concatMap)((t) =>
																					(0, s.concat)(
																						(function (e, t) {
																							return (
																								null !== e && t && t(new ey(e)),
																								(0, s.of)(!0)
																							);
																						})(t.route.parent, r),
																						(function (e, t) {
																							return (
																								null !== e && t && t(new e_(e)),
																								(0, s.of)(!0)
																							);
																						})(t.route, r),
																						(function (e, t, n) {
																							let r = t[t.length - 1],
																								o = t
																									.slice(0, t.length - 1)
																									.reverse()
																									.map((e) =>
																										(function (e) {
																											let t = e.routeConfig
																												? e.routeConfig
																														.canActivateChild
																												: null;
																											return t && 0 !== t.length
																												? { node: e, guards: t }
																												: null;
																										})(e),
																									)
																									.filter((e) => null !== e),
																								i = o.map((t) =>
																									(0, s.defer)(() => {
																										let o = t.guards.map(
																											(o) => {
																												var i, s;
																												let l =
																														null !==
																															(i = e0(
																																t.node,
																															)) && void 0 !== i
																															? i
																															: n,
																													u = e6(o, l);
																												let d =
																													(s = u) &&
																													e9(s.canActivateChild)
																														? u.canActivateChild(
																																r,
																																e,
																														  )
																														: l.runInContext(
																																() => u(r, e),
																														  );
																												return y(d).pipe(
																													(0, a.first)(),
																												);
																											},
																										);
																										return (0, s.of)(o).pipe(
																											tn(),
																										);
																									}),
																								);
																							return (0, s.of)(i).pipe(tn());
																						})(e, t.path, n),
																						(function (e, t, n) {
																							let r = t.routeConfig
																								? t.routeConfig.canActivate
																								: null;
																							if (!r || 0 === r.length)
																								return (0, s.of)(!0);
																							let o = r.map((r) =>
																								(0, s.defer)(() => {
																									var o, i;
																									let s =
																											null !== (o = e0(t)) &&
																											void 0 !== o
																												? o
																												: n,
																										l = e6(r, s);
																									let u =
																										(i = l) && e9(i.canActivate)
																											? l.canActivate(t, e)
																											: s.runInContext(() =>
																													l(t, e),
																											  );
																									return y(u).pipe(
																										(0, a.first)(),
																									);
																								}),
																							);
																							return (0, s.of)(o).pipe(tn());
																						})(e, t.route, n),
																					),
																				),
																				(0, a.first)((e) => !0 !== e, !0),
																			);
																	  })(t, u, n, i)
																	: (0, s.of)(e),
															),
															(0, a.map)((t) =>
																o._(r._({}, e), { guardsResult: t }),
															),
													  );
											})),
											(0, a.tap)((e) => {
												if (
													((t.guardsResult = e.guardsResult), W(e.guardsResult))
												)
													throw eW(this.urlSerializer, e.guardsResult);
												let n = new eh(
													e.id,
													this.urlSerializer.serialize(e.extractedUrl),
													this.urlSerializer.serialize(e.urlAfterRedirects),
													e.targetSnapshot,
													!!e.guardsResult,
												);
												this.events.next(n);
											}),
											(0, a.filter)(
												(t) =>
													!!t.guardsResult ||
													(e.restoreHistory(t),
													this.cancelNavigationTransition(t, "", 3),
													!1),
											),
											tv((t) => {
												if (t.guards.canActivateChecks.length)
													return (0, s.of)(t).pipe(
														(0, a.tap)((e) => {
															let t = new ep(
																e.id,
																this.urlSerializer.serialize(e.extractedUrl),
																this.urlSerializer.serialize(
																	e.urlAfterRedirects,
																),
																e.targetSnapshot,
															);
															this.events.next(t);
														}),
														(0, a.switchMap)((t) => {
															var n, r;
															let o = !1;
															return (0, s.of)(t).pipe(
																((n = e.paramsInheritanceStrategy),
																(r = this.environmentInjector),
																(0, a.mergeMap)((e) => {
																	let {
																		targetSnapshot: t,
																		guards: { canActivateChecks: o },
																	} = e;
																	if (!o.length) return (0, s.of)(e);
																	let i = 0;
																	return (0, s.from)(o).pipe(
																		(0, a.concatMap)((e) =>
																			(function (e, t, n, r) {
																				let o = e.routeConfig,
																					i = e._resolve;
																				return (
																					(null == o ? void 0 : o.title) !==
																						void 0 &&
																						!tg(o) &&
																						(i[c] = o.title),
																					(function (e, t, n, r) {
																						let o = (function (e) {
																							return [
																								...Object.keys(e),
																								...Object.getOwnPropertySymbols(
																									e,
																								),
																							];
																						})(e);
																						if (0 === o.length)
																							return (0, s.of)({});
																						let i = {};
																						return (0, s.from)(o).pipe(
																							(0, a.mergeMap)((o) =>
																								(function (e, t, n, r) {
																									var o;
																									let i =
																											null !== (o = e0(t)) &&
																											void 0 !== o
																												? o
																												: r,
																										s = e6(e, i),
																										l = s.resolve
																											? s.resolve(t, n)
																											: i.runInContext(() =>
																													s(t, n),
																											  );
																									return y(l);
																								})(e[o], t, n, r).pipe(
																									(0, a.first)(),
																									(0, a.tap)((e) => {
																										i[o] = e;
																									}),
																								),
																							),
																							(0, a.takeLast)(1),
																							(0, a.mapTo)(i),
																							(0, a.catchError)((e) =>
																								te(e)
																									? s.EMPTY
																									: (0, s.throwError)(e),
																							),
																						);
																					})(i, e, t, r).pipe(
																						(0, a.map)(
																							(t) => (
																								(e._resolvedData = t),
																								(e.data = eT(e, n).resolve),
																								o &&
																									tg(o) &&
																									(e.data[c] = o.title),
																								null
																							),
																						),
																					)
																				);
																			})(e.route, t, n, r),
																		),
																		(0, a.tap)(() => i++),
																		(0, a.takeLast)(1),
																		(0, a.mergeMap)((t) =>
																			i === o.length ? (0, s.of)(e) : s.EMPTY,
																		),
																	);
																})),
																(0, a.tap)({
																	next: () => (o = !0),
																	complete: () => {
																		!o &&
																			(e.restoreHistory(t),
																			this.cancelNavigationTransition(
																				t,
																				"undefined" == typeof ngDevMode ||
																				ngDevMode
																					? "At least one route resolver didn't emit any value."
																					: "",
																				2,
																			));
																	},
																}),
															);
														}),
														(0, a.tap)((e) => {
															let t = new em(
																e.id,
																this.urlSerializer.serialize(e.extractedUrl),
																this.urlSerializer.serialize(
																	e.urlAfterRedirects,
																),
																e.targetSnapshot,
															);
															this.events.next(t);
														}),
													);
											}),
											tv((e) => {
												let t = (e) => {
													var n;
													let r = [];
													for (let o of ((null === (n = e.routeConfig) ||
													void 0 === n
														? void 0
														: n.loadComponent) &&
														!e.routeConfig._loadedComponent &&
														r.push(
															this.configLoader
																.loadComponent(e.routeConfig)
																.pipe(
																	(0, a.tap)((t) => {
																		e.component = t;
																	}),
																	(0, a.map)(() => void 0),
																),
														),
													e.children))
														r.push(...t(o));
													return r;
												};
												return (0, s.combineLatest)(
													t(e.targetSnapshot.root),
												).pipe((0, a.defaultIfEmpty)(), (0, a.take)(1));
											}),
											tv(() => this.afterPreactivation()),
											(0, a.map)((n) => {
												let i = (function (e, t, n) {
													let r = eH(e, t._root, n ? n._root : void 0);
													return new eA(r, t);
												})(
													e.routeReuseStrategy,
													n.targetSnapshot,
													n.currentRouterState,
												);
												return (t = o._(r._({}, n), { targetRouterState: i }));
											}),
											(0, a.tap)((t) => {
												(e.currentUrlTree = t.urlAfterRedirects),
													(e.rawUrlTree = e.urlHandlingStrategy.merge(
														t.urlAfterRedirects,
														t.rawUrl,
													)),
													(e.routerState = t.targetRouterState),
													"deferred" === e.urlUpdateStrategy &&
														(!t.extras.skipLocationChange &&
															e.setBrowserUrl(e.rawUrlTree, t),
														(e.browserUrlTree = t.urlAfterRedirects));
											}),
											e5(
												this.rootContexts,
												e.routeReuseStrategy,
												(e) => this.events.next(e),
												this.inputBindingEnabled,
											),
											(0, a.take)(1),
											(0, a.tap)({
												next: (t) => {
													var n;
													(l = !0),
														(this.lastSuccessfulNavigation =
															this.currentNavigation),
														(e.navigated = !0),
														this.events.next(
															new el(
																t.id,
																this.urlSerializer.serialize(t.extractedUrl),
																this.urlSerializer.serialize(e.currentUrlTree),
															),
														),
														null === (n = e.titleStrategy) ||
															void 0 === n ||
															n.updateTitle(t.targetRouterState.snapshot),
														t.resolve(!0);
												},
												complete: () => {
													l = !0;
												},
											}),
											(0, a.finalize)(() => {
												var e;
												if (!l && !u) {
													let e =
														"undefined" == typeof ngDevMode || ngDevMode
															? `Navigation ID ${t.id} is not equal to the current navigation id ${this.navigationId}`
															: "";
													this.cancelNavigationTransition(t, e, 1);
												}
												(null === (e = this.currentNavigation) || void 0 === e
													? void 0
													: e.id) === t.id && (this.currentNavigation = null);
											}),
											(0, a.catchError)((n) => {
												var r, o;
												if (((u = !0), (r = n) && r[ez])) {
													!eG(n) &&
														((e.navigated = !0), e.restoreHistory(t, !0));
													let r = new ea(
														t.id,
														this.urlSerializer.serialize(t.extractedUrl),
														n.message,
														n.cancellationCode,
													);
													if ((this.events.next(r), eG(n))) {
														let r = e.urlHandlingStrategy.merge(
																n.url,
																e.rawUrlTree,
															),
															o = {
																skipLocationChange: t.extras.skipLocationChange,
																replaceUrl:
																	"eager" === e.urlUpdateStrategy ||
																	(function (e) {
																		return e !== eo;
																	})(t.source),
															};
														e.scheduleNavigation(r, eo, null, o, {
															resolve: t.resolve,
															reject: t.reject,
															promise: t.promise,
														});
													} else t.resolve(!1);
												} else {
													e.restoreHistory(t, !0);
													let r = new ed(
														t.id,
														this.urlSerializer.serialize(t.extractedUrl),
														n,
														null !== (o = t.targetSnapshot) && void 0 !== o
															? o
															: void 0,
													);
													this.events.next(r);
													try {
														t.resolve(e.errorHandler(n));
													} catch (e) {
														t.reject(e);
													}
												}
												return s.EMPTY;
											}),
										);
									}),
								)
							);
						}
						cancelNavigationTransition(e, t, n) {
							let r = new ea(
								e.id,
								this.urlSerializer.serialize(e.extractedUrl),
								t,
								n,
							);
							this.events.next(r), e.resolve(!1);
						}
						constructor() {
							(this.currentNavigation = null),
								(this.lastSuccessfulNavigation = null),
								(this.events = new s.Subject()),
								(this.configLoader = (0, i.inject)(tb)),
								(this.environmentInjector = (0, i.inject)(
									i.EnvironmentInjector,
								)),
								(this.urlSerializer = (0, i.inject)(E)),
								(this.rootContexts = (0, i.inject)(ew)),
								(this.inputBindingEnabled =
									null !== (0, i.inject)(eB, { optional: !0 })),
								(this.navigationId = 0),
								(this.afterPreactivation = () => (0, s.of)(void 0)),
								(this.rootComponentType = null);
							(this.configLoader.onLoadEndListener = (e) =>
								this.events.next(new ev(e))),
								(this.configLoader.onLoadStartListener = (e) =>
									this.events.next(new eg(e)));
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				function tx(e) {
					return e !== eo;
				}
				"undefined" == typeof ngDevMode || ngDevMode;
				let tD = (() => {
					class e {
						buildTitle(e) {
							let t;
							let n = e.root;
							for (; void 0 !== n; ) {
								var r;
								(t =
									null !== (r = this.getResolvedTitleForRoute(n)) &&
									void 0 !== r
										? r
										: t),
									(n = n.children.find((e) => e.outlet === d));
							}
							return t;
						}
						getResolvedTitleForRoute(e) {
							return e.data[c];
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function () {
								return (0, i.inject)(tw);
							},
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tw = (() => {
					class e extends tD {
						updateTitle(e) {
							let t = this.buildTitle(e);
							void 0 !== t && this.title.setTitle(t);
						}
						constructor(e) {
							super(), (this.title = e);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵinject(u.Title));
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tM = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: function () {
								return (0, i.inject)(tS);
							},
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				class tC {
					shouldDetach(e) {
						return !1;
					}
					store(e, t) {}
					shouldAttach(e) {
						return !1;
					}
					retrieve(e) {
						return null;
					}
					shouldReuseRoute(e, t) {
						return e.routeConfig === t.routeConfig;
					}
				}
				let tS = (() => {
					let e;
					class t extends tC {}
					return (
						(t.ɵfac = function (n) {
							return (e || (e = i.ɵɵgetInheritedFactory(t)))(n || t);
						}),
						(t.ɵprov = i.ɵɵdefineInjectable({
							token: t,
							factory: t.ɵfac,
							providedIn: "root",
						})),
						t
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tE = new i.InjectionToken(
						"undefined" == typeof ngDevMode || ngDevMode ? "router config" : "",
						{ providedIn: "root", factory: () => ({}) },
					),
					tO = (() => {
						class e {}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: function () {
									return (0, i.inject)(tA);
								},
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tA = (() => {
					class e {
						shouldProcessUrl(e) {
							return !0;
						}
						extract(e) {
							return e;
						}
						merge(e, t) {
							return e;
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				var tI =
					(((tI = tI || {})[(tI.COMPLETE = 0)] = "COMPLETE"),
					(tI[(tI.FAILED = 1)] = "FAILED"),
					(tI[(tI.REDIRECTING = 2)] = "REDIRECTING"),
					tI);
				function tP(e, t) {
					e.events
						.pipe(
							(0, a.filter)(
								(e) =>
									e instanceof el ||
									e instanceof ea ||
									e instanceof ed ||
									e instanceof eu,
							),
							(0, a.map)((e) => {
								if (e instanceof el || e instanceof eu) return tI.COMPLETE;
								let t = e instanceof ea && (0 === e.code || 1 === e.code);
								return t ? tI.REDIRECTING : tI.FAILED;
							}),
							(0, a.filter)((e) => e !== tI.REDIRECTING),
							(0, a.take)(1),
						)
						.subscribe(() => {
							t();
						});
				}
				function tT(e) {
					throw e;
				}
				function tk(e, t, n) {
					return t.parse("/");
				}
				let tF = {
						paths: "exact",
						fragment: "ignored",
						matrixParams: "ignored",
						queryParams: "exact",
					},
					tR = {
						paths: "subset",
						fragment: "ignored",
						matrixParams: "ignored",
						queryParams: "subset",
					},
					tN = (() => {
						class e {
							get navigationId() {
								return this.navigationTransitions.navigationId;
							}
							get browserPageId() {
								var e;
								if ("computed" === this.canceledNavigationResolution)
									return null === (e = this.location.getState()) || void 0 === e
										? void 0
										: e.ɵrouterPageId;
							}
							get events() {
								return this.navigationTransitions.events;
							}
							resetRootComponentType(e) {
								(this.routerState.root.component = e),
									(this.navigationTransitions.rootComponentType = e);
							}
							initialNavigation() {
								if (
									(this.setUpLocationChangeListener(),
									!this.navigationTransitions.hasRequestedNavigation)
								) {
									let e = this.location.getState();
									this.navigateToSyncWithBrowser(this.location.path(!0), eo, e);
								}
							}
							setUpLocationChangeListener() {
								!this.locationSubscription &&
									(this.locationSubscription = this.location.subscribe((e) => {
										let t = "popstate" === e.type ? "popstate" : "hashchange";
										"popstate" === t &&
											setTimeout(() => {
												this.navigateToSyncWithBrowser(e.url, t, e.state);
											}, 0);
									}));
							}
							navigateToSyncWithBrowser(e, t, n) {
								let o = { replaceUrl: !0 },
									i = (null == n ? void 0 : n.navigationId) ? n : null;
								if (n) {
									let e = r._({}, n);
									delete e.navigationId,
										delete e.ɵrouterPageId,
										0 !== Object.keys(e).length && (o.state = e);
								}
								let s = this.parseUrl(e);
								this.scheduleNavigation(s, t, i, o);
							}
							get url() {
								return this.serializeUrl(this.currentUrlTree);
							}
							getCurrentNavigation() {
								return this.navigationTransitions.currentNavigation;
							}
							get lastSuccessfulNavigation() {
								return this.navigationTransitions.lastSuccessfulNavigation;
							}
							resetConfig(e) {
								("undefined" == typeof ngDevMode || ngDevMode) && eQ(e),
									(this.config = e.map(eJ)),
									(this.navigated = !1),
									(this.lastSuccessfulId = -1);
							}
							ngOnDestroy() {
								this.dispose();
							}
							dispose() {
								this.navigationTransitions.complete(),
									this.locationSubscription &&
										(this.locationSubscription.unsubscribe(),
										(this.locationSubscription = void 0)),
									(this.disposed = !0);
							}
							createUrlTree(e, t = {}) {
								let n;
								let {
										relativeTo: o,
										queryParams: i,
										fragment: s,
										queryParamsHandling: l,
										preserveFragment: a,
									} = t,
									u = a ? this.currentUrlTree.fragment : s,
									d = null;
								switch (l) {
									case "merge":
										d = r._({}, this.currentUrlTree.queryParams, i);
										break;
									case "preserve":
										d = this.currentUrlTree.queryParams;
										break;
									default:
										d = i || null;
								}
								null !== d && (d = this.removeEmptyProps(d));
								try {
									let e = o ? o.snapshot : this.routerState.snapshot.root;
									n = q(e);
								} catch (t) {
									("string" != typeof e[0] || !e[0].startsWith("/")) &&
										(e = []),
										(n = this.currentUrlTree.root);
								}
								return G(n, e, d, null != u ? u : null);
							}
							navigateByUrl(e, t = { skipLocationChange: !1 }) {
								("undefined" == typeof ngDevMode || ngDevMode) &&
									this.isNgZoneEnabled &&
									!i.NgZone.isInAngularZone() &&
									this.console.warn(
										"Navigation triggered outside Angular zone, did you forget to call 'ngZone.run()'?",
									);
								let n = W(e) ? e : this.parseUrl(e),
									r = this.urlHandlingStrategy.merge(n, this.rawUrlTree);
								return this.scheduleNavigation(r, eo, null, t);
							}
							navigate(e, t = { skipLocationChange: !1 }) {
								return (
									(function (e) {
										for (let t = 0; t < e.length; t++) {
											let n = e[t];
											if (null == n)
												throw new i.ɵRuntimeError(
													4008,
													("undefined" == typeof ngDevMode || ngDevMode) &&
														`The requested path contains ${n} segment at index ${t}`,
												);
										}
									})(e),
									this.navigateByUrl(this.createUrlTree(e, t), t)
								);
							}
							serializeUrl(e) {
								return this.urlSerializer.serialize(e);
							}
							parseUrl(e) {
								let t;
								try {
									t = this.urlSerializer.parse(e);
								} catch (n) {
									t = this.malformedUriErrorHandler(n, this.urlSerializer, e);
								}
								return t;
							}
							isActive(e, t) {
								let n;
								if (
									((n = !0 === t ? r._({}, tF) : !1 === t ? r._({}, tR) : t),
									W(e))
								)
									return j(this.currentUrlTree, e, n);
								let o = this.parseUrl(e);
								return j(this.currentUrlTree, o, n);
							}
							removeEmptyProps(e) {
								return Object.keys(e).reduce((t, n) => {
									let r = e[n];
									return null != r && (t[n] = r), t;
								}, {});
							}
							scheduleNavigation(e, t, n, r, o) {
								let i, s, l;
								if (this.disposed) return Promise.resolve(!1);
								o
									? ((i = o.resolve), (s = o.reject), (l = o.promise))
									: (l = new Promise((e, t) => {
											(i = e), (s = t);
									  }));
								let a = this.pendingTasks.add();
								return (
									tP(this, () => {
										Promise.resolve().then(() => this.pendingTasks.remove(a));
									}),
									this.navigationTransitions.handleNavigationRequest({
										source: t,
										restoredState: n,
										currentUrlTree: this.currentUrlTree,
										currentRawUrl: this.currentUrlTree,
										rawUrl: e,
										extras: r,
										resolve: i,
										reject: s,
										promise: l,
										currentSnapshot: this.routerState.snapshot,
										currentRouterState: this.routerState,
									}),
									l.catch((e) => Promise.reject(e))
								);
							}
							setBrowserUrl(e, t) {
								let n = this.urlSerializer.serialize(e);
								if (
									this.location.isCurrentPathEqualTo(n) ||
									t.extras.replaceUrl
								) {
									let e = this.browserPageId,
										o = r._(
											{},
											t.extras.state,
											this.generateNgRouterState(t.id, e),
										);
									this.location.replaceState(n, "", o);
								} else {
									var o;
									let e = r._(
										{},
										t.extras.state,
										this.generateNgRouterState(
											t.id,
											(null !== (o = this.browserPageId) && void 0 !== o
												? o
												: 0) + 1,
										),
									);
									this.location.go(n, "", e);
								}
							}
							restoreHistory(e, t = !1) {
								if ("computed" === this.canceledNavigationResolution) {
									var n, r;
									let t =
											null !== (r = this.browserPageId) && void 0 !== r
												? r
												: this.currentPageId,
										o = this.currentPageId - t;
									0 !== o
										? this.location.historyGo(o)
										: this.currentUrlTree ===
												(null === (n = this.getCurrentNavigation()) ||
												void 0 === n
													? void 0
													: n.finalUrl) &&
										  0 === o &&
										  (this.resetState(e),
										  (this.browserUrlTree = e.currentUrlTree),
										  this.resetUrlToCurrentUrlTree());
								} else
									"replace" === this.canceledNavigationResolution &&
										(t && this.resetState(e), this.resetUrlToCurrentUrlTree());
							}
							resetState(e) {
								(this.routerState = e.currentRouterState),
									(this.currentUrlTree = e.currentUrlTree),
									(this.rawUrlTree = this.urlHandlingStrategy.merge(
										this.currentUrlTree,
										e.rawUrl,
									));
							}
							resetUrlToCurrentUrlTree() {
								this.location.replaceState(
									this.urlSerializer.serialize(this.rawUrlTree),
									"",
									this.generateNgRouterState(
										this.lastSuccessfulId,
										this.currentPageId,
									),
								);
							}
							generateNgRouterState(e, t) {
								return "computed" === this.canceledNavigationResolution
									? { navigationId: e, ɵrouterPageId: t }
									: { navigationId: e };
							}
							constructor() {
								var e, t;
								(this.disposed = !1),
									(this.currentPageId = 0),
									(this.console = (0, i.inject)(i.ɵConsole)),
									(this.isNgZoneEnabled = !1),
									(this.options = (0, i.inject)(tE, { optional: !0 }) || {}),
									(this.pendingTasks = (0, i.inject)(
										i.ɵInitialRenderPendingTasks,
									)),
									(this.errorHandler = this.options.errorHandler || tT),
									(this.malformedUriErrorHandler =
										this.options.malformedUriErrorHandler || tk),
									(this.navigated = !1),
									(this.lastSuccessfulId = -1),
									(this.urlHandlingStrategy = (0, i.inject)(tO)),
									(this.routeReuseStrategy = (0, i.inject)(tM)),
									(this.titleStrategy = (0, i.inject)(tD)),
									(this.onSameUrlNavigation =
										this.options.onSameUrlNavigation || "ignore"),
									(this.paramsInheritanceStrategy =
										this.options.paramsInheritanceStrategy || "emptyOnly"),
									(this.urlUpdateStrategy =
										this.options.urlUpdateStrategy || "deferred"),
									(this.canceledNavigationResolution =
										this.options.canceledNavigationResolution || "replace"),
									(this.config =
										null !==
											(t =
												null === (e = (0, i.inject)(ty, { optional: !0 })) ||
												void 0 === e
													? void 0
													: e.flat()) && void 0 !== t
											? t
											: []),
									(this.navigationTransitions = (0, i.inject)(tj)),
									(this.urlSerializer = (0, i.inject)(E)),
									(this.location = (0, i.inject)(l.Location)),
									(this.componentInputBindingEnabled = !!(0, i.inject)(eB, {
										optional: !0,
									})),
									(this.isNgZoneEnabled =
										(0, i.inject)(i.NgZone) instanceof i.NgZone &&
										i.NgZone.isInAngularZone()),
									this.resetConfig(this.config),
									(this.currentUrlTree = new w()),
									(this.rawUrlTree = this.currentUrlTree),
									(this.browserUrlTree = this.currentUrlTree),
									(this.routerState = eI(this.currentUrlTree, null)),
									this.navigationTransitions.setupNavigations(this).subscribe(
										(e) => {
											var t;
											(this.lastSuccessfulId = e.id),
												(this.currentPageId =
													null !== (t = this.browserPageId) && void 0 !== t
														? t
														: 0);
										},
										(e) => {
											this.console.warn(`Unhandled Navigation Error: ${e}`);
										},
									);
							}
						}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							})),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tL = (() => {
					class e {
						set preserveFragment(e) {
							this._preserveFragment = (0, i.ɵcoerceToBoolean)(e);
						}
						get preserveFragment() {
							return this._preserveFragment;
						}
						set skipLocationChange(e) {
							this._skipLocationChange = (0, i.ɵcoerceToBoolean)(e);
						}
						get skipLocationChange() {
							return this._skipLocationChange;
						}
						set replaceUrl(e) {
							this._replaceUrl = (0, i.ɵcoerceToBoolean)(e);
						}
						get replaceUrl() {
							return this._replaceUrl;
						}
						setTabIndexIfNotOnNativeEl(e) {
							null == this.tabIndexAttribute &&
								!this.isAnchorElement &&
								this.applyAttributeValue("tabindex", e);
						}
						ngOnChanges(e) {
							this.isAnchorElement && this.updateHref(),
								this.onChanges.next(this);
						}
						set routerLink(e) {
							null != e
								? ((this.commands = Array.isArray(e) ? e : [e]),
								  this.setTabIndexIfNotOnNativeEl("0"))
								: ((this.commands = null),
								  this.setTabIndexIfNotOnNativeEl(null));
						}
						onClick(e, t, n, r, o) {
							if (
								null === this.urlTree ||
								(this.isAnchorElement &&
									(0 !== e ||
										t ||
										n ||
										r ||
										o ||
										("string" == typeof this.target && "_self" != this.target)))
							)
								return !0;
							let i = {
								skipLocationChange: this.skipLocationChange,
								replaceUrl: this.replaceUrl,
								state: this.state,
							};
							return (
								this.router.navigateByUrl(this.urlTree, i),
								!this.isAnchorElement
							);
						}
						ngOnDestroy() {
							var e;
							null === (e = this.subscription) ||
								void 0 === e ||
								e.unsubscribe();
						}
						updateHref() {
							var e;
							this.href =
								null !== this.urlTree && this.locationStrategy
									? null === (e = this.locationStrategy) || void 0 === e
										? void 0
										: e.prepareExternalUrl(
												this.router.serializeUrl(this.urlTree),
										  )
									: null;
							let t =
								null === this.href
									? null
									: (0, i.ɵɵsanitizeUrlOrResourceUrl)(
											this.href,
											this.el.nativeElement.tagName.toLowerCase(),
											"href",
									  );
							this.applyAttributeValue("href", t);
						}
						applyAttributeValue(e, t) {
							let n = this.renderer,
								r = this.el.nativeElement;
							null !== t ? n.setAttribute(r, e, t) : n.removeAttribute(r, e);
						}
						get urlTree() {
							return null === this.commands
								? null
								: this.router.createUrlTree(this.commands, {
										relativeTo:
											void 0 !== this.relativeTo ? this.relativeTo : this.route,
										queryParams: this.queryParams,
										fragment: this.fragment,
										queryParamsHandling: this.queryParamsHandling,
										preserveFragment: this.preserveFragment,
								  });
						}
						constructor(e, t, n, r, o, i) {
							var l;
							(this.router = e),
								(this.route = t),
								(this.tabIndexAttribute = n),
								(this.renderer = r),
								(this.el = o),
								(this.locationStrategy = i),
								(this._preserveFragment = !1),
								(this._skipLocationChange = !1),
								(this._replaceUrl = !1),
								(this.href = null),
								(this.commands = null),
								(this.onChanges = new s.Subject());
							let a =
								null === (l = o.nativeElement.tagName) || void 0 === l
									? void 0
									: l.toLowerCase();
							(this.isAnchorElement = "a" === a || "area" === a),
								this.isAnchorElement
									? (this.subscription = e.events.subscribe((e) => {
											e instanceof el && this.updateHref();
									  }))
									: this.setTabIndexIfNotOnNativeEl("0");
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(tN),
								i.ɵɵdirectiveInject(eP),
								i.ɵɵinjectAttribute("tabindex"),
								i.ɵɵdirectiveInject(i.Renderer2),
								i.ɵɵdirectiveInject(i.ElementRef),
								i.ɵɵdirectiveInject(l.LocationStrategy),
							);
						}),
						(e.ɵdir = i.ɵɵdefineDirective({
							type: e,
							selectors: [["", "routerLink", ""]],
							hostVars: 1,
							hostBindings: function (e, t) {
								1 & e &&
									i.ɵɵlistener("click", function (e) {
										return t.onClick(
											e.button,
											e.ctrlKey,
											e.shiftKey,
											e.altKey,
											e.metaKey,
										);
									}),
									2 & e && i.ɵɵattribute("target", t.target);
							},
							inputs: {
								target: "target",
								queryParams: "queryParams",
								fragment: "fragment",
								queryParamsHandling: "queryParamsHandling",
								state: "state",
								relativeTo: "relativeTo",
								preserveFragment: "preserveFragment",
								skipLocationChange: "skipLocationChange",
								replaceUrl: "replaceUrl",
								routerLink: "routerLink",
							},
							standalone: !0,
							features: [i.ɵɵNgOnChangesFeature],
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							get isActive() {
								return this._isActive;
							}
							ngAfterContentInit() {
								(0, s.of)(this.links.changes, (0, s.of)(null))
									.pipe((0, a.mergeAll)())
									.subscribe((e) => {
										this.update(), this.subscribeToEachLinkOnChanges();
									});
							}
							subscribeToEachLinkOnChanges() {
								var e;
								null === (e = this.linkInputChangesSubscription) ||
									void 0 === e ||
									e.unsubscribe();
								let t = [...this.links.toArray(), this.link]
									.filter((e) => !!e)
									.map((e) => e.onChanges);
								this.linkInputChangesSubscription = (0, s.from)(t)
									.pipe((0, a.mergeAll)())
									.subscribe((e) => {
										this._isActive !== this.isLinkActive(this.router)(e) &&
											this.update();
									});
							}
							set routerLinkActive(e) {
								let t = Array.isArray(e) ? e : e.split(" ");
								this.classes = t.filter((e) => !!e);
							}
							ngOnChanges(e) {
								this.update();
							}
							ngOnDestroy() {
								var e;
								this.routerEventsSubscription.unsubscribe(),
									null === (e = this.linkInputChangesSubscription) ||
										void 0 === e ||
										e.unsubscribe();
							}
							update() {
								this.links &&
									this.router.navigated &&
									Promise.resolve().then(() => {
										let e = this.hasActiveLinks();
										this._isActive !== e &&
											((this._isActive = e),
											this.cdr.markForCheck(),
											this.classes.forEach((t) => {
												e
													? this.renderer.addClass(
															this.element.nativeElement,
															t,
													  )
													: this.renderer.removeClass(
															this.element.nativeElement,
															t,
													  );
											}),
											e && void 0 !== this.ariaCurrentWhenActive
												? this.renderer.setAttribute(
														this.element.nativeElement,
														"aria-current",
														this.ariaCurrentWhenActive.toString(),
												  )
												: this.renderer.removeAttribute(
														this.element.nativeElement,
														"aria-current",
												  ),
											this.isActiveChange.emit(e));
									});
							}
							isLinkActive(e) {
								let t = (function (e) {
									return !!e.paths;
								})(this.routerLinkActiveOptions)
									? this.routerLinkActiveOptions
									: this.routerLinkActiveOptions.exact || !1;
								return (n) => !!n.urlTree && e.isActive(n.urlTree, t);
							}
							hasActiveLinks() {
								let e = this.isLinkActive(this.router);
								return (this.link && e(this.link)) || this.links.some(e);
							}
							constructor(e, t, n, r, o) {
								(this.router = e),
									(this.element = t),
									(this.renderer = n),
									(this.cdr = r),
									(this.link = o),
									(this.classes = []),
									(this._isActive = !1),
									(this.routerLinkActiveOptions = { exact: !1 }),
									(this.isActiveChange = new i.EventEmitter()),
									(this.routerEventsSubscription = e.events.subscribe((e) => {
										e instanceof el && this.update();
									}));
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵdirectiveInject(tN),
								i.ɵɵdirectiveInject(i.ElementRef),
								i.ɵɵdirectiveInject(i.Renderer2),
								i.ɵɵdirectiveInject(i.ChangeDetectorRef),
								i.ɵɵdirectiveInject(tL, 8),
							);
						}),
							(e.ɵdir = i.ɵɵdefineDirective({
								type: e,
								selectors: [["", "routerLinkActive", ""]],
								contentQueries: function (e, t, n) {
									if ((1 & e && i.ɵɵcontentQuery(n, tL, 5), 2 & e)) {
										let e;
										i.ɵɵqueryRefresh((e = i.ɵɵloadQuery())) && (t.links = e);
									}
								},
								inputs: {
									routerLinkActiveOptions: "routerLinkActiveOptions",
									ariaCurrentWhenActive: "ariaCurrentWhenActive",
									routerLinkActive: "routerLinkActive",
								},
								outputs: { isActiveChange: "isActiveChange" },
								exportAs: ["routerLinkActive"],
								standalone: !0,
								features: [i.ɵɵNgOnChangesFeature],
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				class t$ {}
				(() => {
					class e {
						preload(e, t) {
							return t().pipe((0, a.catchError)(() => (0, s.of)(null)));
						}
					}
					(e.ɵfac = function (t) {
						return new (t || e)();
					}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						}));
				})(),
					"undefined" == typeof ngDevMode || ngDevMode,
					(() => {
						class e {
							preload(e, t) {
								return (0, s.of)(null);
							}
						}
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
							(e.ɵprov = i.ɵɵdefineInjectable({
								token: e,
								factory: e.ɵfac,
								providedIn: "root",
							}));
					})(),
					"undefined" == typeof ngDevMode || ngDevMode;
				let tV = (() => {
					class e {
						setUpPreloading() {
							this.subscription = this.router.events
								.pipe(
									(0, a.filter)((e) => e instanceof el),
									(0, a.concatMap)(() => this.preload()),
								)
								.subscribe(() => {});
						}
						preload() {
							return this.processRoutes(this.injector, this.router.config);
						}
						ngOnDestroy() {
							this.subscription && this.subscription.unsubscribe();
						}
						processRoutes(e, t) {
							let n = [];
							for (let s of t) {
								var r, o, l;
								s.providers &&
									!s._injector &&
									(s._injector = (0, i.createEnvironmentInjector)(
										s.providers,
										e,
										`Route: ${s.path}`,
									));
								let t = null !== (r = s._injector) && void 0 !== r ? r : e,
									a = null !== (o = s._loadedInjector) && void 0 !== o ? o : t;
								((s.loadChildren && !s._loadedRoutes && void 0 === s.canLoad) ||
									(s.loadComponent && !s._loadedComponent)) &&
									n.push(this.preloadConfig(t, s)),
									(s.children || s._loadedRoutes) &&
										n.push(
											this.processRoutes(
												a,
												null !== (l = s.children) && void 0 !== l
													? l
													: s._loadedRoutes,
											),
										);
							}
							return (0, s.from)(n).pipe((0, a.mergeAll)());
						}
						preloadConfig(e, t) {
							return this.preloadingStrategy.preload(t, () => {
								let n;
								n =
									t.loadChildren && void 0 === t.canLoad
										? this.loader.loadChildren(e, t)
										: (0, s.of)(null);
								let r = n.pipe(
									(0, a.mergeMap)((n) => {
										var r;
										return null === n
											? (0, s.of)(void 0)
											: ((t._loadedRoutes = n.routes),
											  (t._loadedInjector = n.injector),
											  this.processRoutes(
													null !== (r = n.injector) && void 0 !== r ? r : e,
													n.routes,
											  ));
									}),
								);
								if (!t.loadComponent || t._loadedComponent) return r;
								{
									let e = this.loader.loadComponent(t);
									return (0, s.from)([r, e]).pipe((0, a.mergeAll)());
								}
							});
						}
						constructor(e, t, n, r, o) {
							(this.router = e),
								(this.injector = n),
								(this.preloadingStrategy = r),
								(this.loader = o);
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(
								i.ɵɵinject(tN),
								i.ɵɵinject(i.Compiler),
								i.ɵɵinject(i.EnvironmentInjector),
								i.ɵɵinject(t$),
								i.ɵɵinject(tb),
							);
						}),
						(e.ɵprov = i.ɵɵdefineInjectable({
							token: e,
							factory: e.ɵfac,
							providedIn: "root",
						})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				let tB = new i.InjectionToken(""),
					tU = (() => {
						class e {
							init() {
								"disabled" !== this.options.scrollPositionRestoration &&
									this.viewportScroller.setHistoryScrollRestoration("manual"),
									(this.routerEventsSubscription = this.createScrollEvents()),
									(this.scrollEventsSubscription = this.consumeScrollEvents());
							}
							createScrollEvents() {
								return this.transitions.events.subscribe((e) => {
									e instanceof es
										? ((this.store[this.lastId] =
												this.viewportScroller.getScrollPosition()),
										  (this.lastSource = e.navigationTrigger),
										  (this.restoredId = e.restoredState
												? e.restoredState.navigationId
												: 0))
										: e instanceof el
										? ((this.lastId = e.id),
										  this.scheduleScrollEvent(
												e,
												this.urlSerializer.parse(e.urlAfterRedirects).fragment,
										  ))
										: e instanceof eu &&
										  0 === e.code &&
										  ((this.lastSource = void 0),
										  (this.restoredId = 0),
										  this.scheduleScrollEvent(
												e,
												this.urlSerializer.parse(e.url).fragment,
										  ));
								});
							}
							consumeScrollEvents() {
								return this.transitions.events.subscribe((e) => {
									e instanceof ex &&
										(e.position
											? "top" === this.options.scrollPositionRestoration
												? this.viewportScroller.scrollToPosition([0, 0])
												: "enabled" ===
														this.options.scrollPositionRestoration &&
												  this.viewportScroller.scrollToPosition(e.position)
											: e.anchor && "enabled" === this.options.anchorScrolling
											? this.viewportScroller.scrollToAnchor(e.anchor)
											: "disabled" !== this.options.scrollPositionRestoration &&
											  this.viewportScroller.scrollToPosition([0, 0]));
								});
							}
							scheduleScrollEvent(e, t) {
								this.zone.runOutsideAngular(() => {
									setTimeout(() => {
										this.zone.run(() => {
											this.transitions.events.next(
												new ex(
													e,
													"popstate" === this.lastSource
														? this.store[this.restoredId]
														: null,
													t,
												),
											);
										});
									}, 0);
								});
							}
							ngOnDestroy() {
								var e, t;
								null === (e = this.routerEventsSubscription) ||
									void 0 === e ||
									e.unsubscribe(),
									null === (t = this.scrollEventsSubscription) ||
										void 0 === t ||
										t.unsubscribe();
							}
							constructor(e, t, n, r, o = {}) {
								(this.urlSerializer = e),
									(this.transitions = t),
									(this.viewportScroller = n),
									(this.zone = r),
									(this.options = o),
									(this.lastId = 0),
									(this.lastSource = "imperative"),
									(this.restoredId = 0),
									(this.store = {}),
									(o.scrollPositionRestoration =
										o.scrollPositionRestoration || "disabled"),
									(o.anchorScrolling = o.anchorScrolling || "disabled");
							}
						}
						return (
							(e.ɵfac = function (e) {
								i.ɵɵinvalidFactory();
							}),
							(e.ɵprov = i.ɵɵdefineInjectable({ token: e, factory: e.ɵfac })),
							e
						);
					})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function tH(e, t) {
					return { ɵkind: e, ɵproviders: t };
				}
				let tz = new i.InjectionToken("", {
					providedIn: "root",
					factory: () => !1,
				});
				function tW() {
					let e = (0, i.inject)(i.Injector);
					return (t) => {
						var n, r;
						let o = e.get(i.ApplicationRef);
						if (t !== o.components[0]) return;
						let s = e.get(tN),
							l = e.get(tq);
						1 === e.get(tG) && s.initialNavigation(),
							null === (n = e.get(tZ, null, i.InjectFlags.Optional)) ||
								void 0 === n ||
								n.setUpPreloading(),
							null === (r = e.get(tB, null, i.InjectFlags.Optional)) ||
								void 0 === r ||
								r.init(),
							s.resetRootComponentType(o.componentTypes[0]),
							!l.closed && (l.next(), l.complete(), l.unsubscribe());
					};
				}
				i.ENVIRONMENT_INITIALIZER,
					() => () => {
						!(0, i.inject)(tz) &&
							console.warn(
								"`provideRoutes` was called without `provideRouter` or `RouterModule.forRoot`. This is likely a mistake.",
							);
					};
				let tq = new i.InjectionToken(
						"undefined" == typeof ngDevMode || ngDevMode
							? "bootstrap done indicator"
							: "",
						{ factory: () => new s.Subject() },
					),
					tG = new i.InjectionToken(
						"undefined" == typeof ngDevMode || ngDevMode
							? "initial navigation"
							: "",
						{ providedIn: "root", factory: () => 1 },
					),
					tZ = new i.InjectionToken(
						"undefined" == typeof ngDevMode || ngDevMode
							? "router preloader"
							: "",
					),
					tY = new i.InjectionToken(
						"undefined" == typeof ngDevMode || ngDevMode
							? "router duplicate forRoot guard"
							: "ROUTER_FORROOT_GUARD",
					),
					tQ = [
						l.Location,
						{ provide: E, useClass: O },
						tN,
						ew,
						{
							provide: eP,
							useFactory: function (e) {
								return e.routerState.root;
							},
							deps: [tN],
						},
						tb,
						"undefined" == typeof ngDevMode || ngDevMode
							? { provide: tz, useValue: !0 }
							: [],
					];
				function tK() {
					return new i.NgProbeToken("Router", tN);
				}
				let tJ = (() => {
					class e {
						static forRoot(t, n) {
							let r;
							return {
								ngModule: e,
								providers: [
									tQ,
									("undefined" == typeof ngDevMode || ngDevMode) &&
									(null == n ? void 0 : n.enableTracing)
										? ((r = []),
										  tH(
												1,
												(r =
													"undefined" == typeof ngDevMode || ngDevMode
														? [
																{
																	provide: i.ENVIRONMENT_INITIALIZER,
																	multi: !0,
																	useFactory: () => {
																		let e = (0, i.inject)(tN);
																		return () =>
																			e.events.subscribe((e) => {
																				var t, n;
																				null === (t = console.group) ||
																					void 0 === t ||
																					t.call(
																						console,
																						`Router Event: ${e.constructor.name}`,
																					),
																					console.log(
																						(function (e) {
																							var t, n, r, o;
																							switch (e.type) {
																								case 14:
																									return `ActivationEnd(path: '${
																										(null ===
																											(t =
																												e.snapshot
																													.routeConfig) ||
																										void 0 === t
																											? void 0
																											: t.path) || ""
																									}')`;
																								case 13:
																									return `ActivationStart(path: '${
																										(null ===
																											(n =
																												e.snapshot
																													.routeConfig) ||
																										void 0 === n
																											? void 0
																											: n.path) || ""
																									}')`;
																								case 12:
																									return `ChildActivationEnd(path: '${
																										(null ===
																											(r =
																												e.snapshot
																													.routeConfig) ||
																										void 0 === r
																											? void 0
																											: r.path) || ""
																									}')`;
																								case 11:
																									return `ChildActivationStart(path: '${
																										(null ===
																											(o =
																												e.snapshot
																													.routeConfig) ||
																										void 0 === o
																											? void 0
																											: o.path) || ""
																									}')`;
																								case 8:
																									return `GuardsCheckEnd(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}', state: ${e.state}, shouldActivate: ${e.shouldActivate})`;
																								case 7:
																									return `GuardsCheckStart(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}', state: ${e.state})`;
																								case 2:
																									return `NavigationCancel(id: ${e.id}, url: '${e.url}')`;
																								case 16:
																									return `NavigationSkipped(id: ${e.id}, url: '${e.url}')`;
																								case 1:
																									return `NavigationEnd(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}')`;
																								case 3:
																									return `NavigationError(id: ${e.id}, url: '${e.url}', error: ${e.error})`;
																								case 0:
																									return `NavigationStart(id: ${e.id}, url: '${e.url}')`;
																								case 6:
																									return `ResolveEnd(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}', state: ${e.state})`;
																								case 5:
																									return `ResolveStart(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}', state: ${e.state})`;
																								case 10:
																									return `RouteConfigLoadEnd(path: ${e.route.path})`;
																								case 9:
																									return `RouteConfigLoadStart(path: ${e.route.path})`;
																								case 4:
																									return `RoutesRecognized(id: ${e.id}, url: '${e.url}', urlAfterRedirects: '${e.urlAfterRedirects}', state: ${e.state})`;
																								case 15:
																									let i = e.position
																										? `${e.position[0]}, ${e.position[1]}`
																										: null;
																									return `Scroll(anchor: '${e.anchor}', position: '${i}')`;
																							}
																						})(e),
																					),
																					console.log(e),
																					null === (n = console.groupEnd) ||
																						void 0 === n ||
																						n.call(console);
																			});
																	},
																},
														  ]
														: []),
										  )).ɵproviders
										: [],
									{ provide: ty, multi: !0, useValue: t },
									{
										provide: tY,
										useFactory: tX,
										deps: [[tN, new i.Optional(), new i.SkipSelf()]],
									},
									{ provide: tE, useValue: n || {} },
									(null == n ? void 0 : n.useHash)
										? (function () {
												return {
													provide: l.LocationStrategy,
													useClass: l.HashLocationStrategy,
												};
										  })()
										: (function () {
												return {
													provide: l.LocationStrategy,
													useClass: l.PathLocationStrategy,
												};
										  })(),
									(function () {
										return {
											provide: tB,
											useFactory: () => {
												let e = (0, i.inject)(l.ViewportScroller),
													t = (0, i.inject)(i.NgZone),
													n = (0, i.inject)(tE),
													r = (0, i.inject)(tj),
													o = (0, i.inject)(E);
												return (
													n.scrollOffset && e.setOffset(n.scrollOffset),
													new tU(o, r, e, t, n)
												);
											},
										};
									})(),
									(null == n ? void 0 : n.preloadingStrategy)
										? (function (e) {
												let t = [
													{ provide: tZ, useExisting: tV },
													{ provide: t$, useExisting: e },
												];
												return tH(0, t);
										  })(n.preloadingStrategy).ɵproviders
										: [],
									{ provide: i.NgProbeToken, multi: !0, useFactory: tK },
									(null == n ? void 0 : n.initialNavigation)
										? (function (e) {
												return [
													"disabled" === e.initialNavigation
														? (function () {
																let e = [
																	{
																		provide: i.APP_INITIALIZER,
																		multi: !0,
																		useFactory: () => {
																			let e = (0, i.inject)(tN);
																			return () => {
																				e.setUpLocationChangeListener();
																			};
																		},
																	},
																	{ provide: tG, useValue: 2 },
																];
																return tH(3, e);
														  })().ɵproviders
														: [],
													"enabledBlocking" === e.initialNavigation
														? (function () {
																let e = [
																	{ provide: tG, useValue: 0 },
																	{
																		provide: i.APP_INITIALIZER,
																		multi: !0,
																		deps: [i.Injector],
																		useFactory: (e) => {
																			let t = e.get(
																				l.LOCATION_INITIALIZED,
																				Promise.resolve(),
																			);
																			return () =>
																				t.then(
																					() =>
																						new Promise((t) => {
																							let n = e.get(tN),
																								r = e.get(tq);
																							tP(n, () => {
																								t(!0);
																							}),
																								(e.get(tj).afterPreactivation =
																									() => (
																										t(!0),
																										r.closed
																											? (0, s.of)(void 0)
																											: r
																									)),
																								n.initialNavigation();
																						}),
																				);
																		},
																	},
																];
																return tH(2, e);
														  })().ɵproviders
														: [],
												];
										  })(n)
										: [],
									(null == n ? void 0 : n.bindToComponentInputs)
										? (function () {
												let e = [eU, { provide: eB, useExisting: eU }];
												return tH(8, e);
										  })().ɵproviders
										: [],
									(function () {
										return [
											{ provide: t0, useFactory: tW },
											{
												provide: i.APP_BOOTSTRAP_LISTENER,
												multi: !0,
												useExisting: t0,
											},
										];
									})(),
								],
							};
						}
						static forChild(t) {
							return {
								ngModule: e,
								providers: [{ provide: ty, multi: !0, useValue: t }],
							};
						}
						constructor(e) {}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)(i.ɵɵinject(tY, 8));
						}),
						(e.ɵmod = i.ɵɵdefineNgModule({ type: e })),
						(e.ɵinj = i.ɵɵdefineInjector({})),
						e
					);
				})();
				"undefined" == typeof ngDevMode || ngDevMode;
				function tX(e) {
					if (("undefined" == typeof ngDevMode || ngDevMode) && e)
						throw new i.ɵRuntimeError(
							4007,
							"The Router was provided more than once. This can happen if 'forRoot' is used outside of the root injector. Lazy loaded modules should use RouterModule.forChild() instead.",
						);
					return "guarded";
				}
				let t0 = new i.InjectionToken(
					"undefined" == typeof ngDevMode || ngDevMode
						? "Router Initializer"
						: "",
				);
				new i.Version("16.0.0");
			},
			"../../node_modules/@swc/helpers/esm/_define_property.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				function r(e, t, n) {
					return (
						t in e
							? Object.defineProperty(e, t, {
									value: n,
									enumerable: !0,
									configurable: !0,
									writable: !0,
							  })
							: (e[t] = n),
						e
					);
				}
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "_define_property", {
						enumerable: !0,
						get: function () {
							return r;
						},
					});
			},
			"../../node_modules/@swc/helpers/esm/_object_spread.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					_object_spread: function () {
						return o;
					},
					_: function () {
						return o;
					},
				});
				var r = n("../../node_modules/@swc/helpers/esm/_define_property.js");
				function o(e) {
					for (var t = 1; t < arguments.length; t++) {
						var n = null != arguments[t] ? arguments[t] : {},
							o = Object.keys(n);
						"function" == typeof Object.getOwnPropertySymbols &&
							(o = o.concat(
								Object.getOwnPropertySymbols(n).filter(function (e) {
									return Object.getOwnPropertyDescriptor(n, e).enumerable;
								}),
							)),
							o.forEach(function (t) {
								(0, r._define_property)(e, t, n[t]);
							});
					}
					return e;
				}
			},
			"../../node_modules/@swc/helpers/esm/_object_spread_props.js": function (
				e,
				t,
				n,
			) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				!(function (e, t) {
					for (var n in t)
						Object.defineProperty(e, n, { enumerable: !0, get: t[n] });
				})(t, {
					_object_spread_props: function () {
						return r;
					},
					_: function () {
						return r;
					},
				});
				function r(e, t) {
					return (
						(t = null != t ? t : {}),
						Object.getOwnPropertyDescriptors
							? Object.defineProperties(e, Object.getOwnPropertyDescriptors(t))
							: (function (e, t) {
									var n = Object.keys(e);
									if (Object.getOwnPropertySymbols) {
										var r = Object.getOwnPropertySymbols(e);
										n.push.apply(n, r);
									}
									return n;
							  })(Object(t)).forEach(function (n) {
									Object.defineProperty(
										e,
										n,
										Object.getOwnPropertyDescriptor(t, n),
									);
							  }),
						e
					);
				}
			},
			"./src/app/app-routing.module.ts": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "AppRoutingModule", {
						enumerable: !0,
						get: function () {
							return s;
						},
					});
				var r = n("../../node_modules/@angular/router/fesm2022/router.mjs"),
					o = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs"));
				let i = [],
					s = (() => {
						class e {}
						return (
							(e.ɵfac = function (t) {
								return new (t || e)();
							}),
							(e.ɵmod = o.ɵɵdefineNgModule({ type: e })),
							(e.ɵinj = o.ɵɵdefineInjector({
								imports: [r.RouterModule.forRoot(i), r.RouterModule],
							})),
							e
						);
					})();
			},
			"./src/app/app.component.ts": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "AppComponent", {
						enumerable: !0,
						get: function () {
							return f;
						},
					});
				var r = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs")),
					o = n.ir(n("../../node_modules/@angular/common/fesm2022/common.mjs")),
					i = n.ir(n("../../node_modules/@angular/router/fesm2022/router.mjs"));
				function s(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng generate component xyz"),
						r.ɵɵelementEnd());
				}
				function l(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng add @angular/material"),
						r.ɵɵelementEnd());
				}
				function a(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng add @angular/pwa"),
						r.ɵɵelementEnd());
				}
				function u(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng add _____"),
						r.ɵɵelementEnd());
				}
				function d(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng test"),
						r.ɵɵelementEnd());
				}
				function c(e, t) {
					1 & e &&
						(r.ɵɵelementStart(0, "pre"),
						r.ɵɵtext(1, "ng build"),
						r.ɵɵelementEnd());
				}
				let f = (() => {
					class e {
						constructor() {
							this.title = "rspack-ngs";
						}
					}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵcmp = r.ɵɵdefineComponent({
							type: e,
							selectors: [["app-root"]],
							decls: 156,
							vars: 7,
							consts: [
								["role", "banner", 1, "toolbar"],
								[
									"width",
									"40",
									"alt",
									"Angular Logo",
									"src",
									"data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNTAgMjUwIj4KICAgIDxwYXRoIGZpbGw9IiNERDAwMzEiIGQ9Ik0xMjUgMzBMMzEuOSA2My4ybDE0LjIgMTIzLjFMMTI1IDIzMGw3OC45LTQzLjcgMTQuMi0xMjMuMXoiIC8+CiAgICA8cGF0aCBmaWxsPSIjQzMwMDJGIiBkPSJNMTI1IDMwdjIyLjItLjFWMjMwbDc4LjktNDMuNyAxNC4yLTEyMy4xTDEyNSAzMHoiIC8+CiAgICA8cGF0aCAgZmlsbD0iI0ZGRkZGRiIgZD0iTTEyNSA1Mi4xTDY2LjggMTgyLjZoMjEuN2wxMS43LTI5LjJoNDkuNGwxMS43IDI5LjJIMTgzTDEyNSA1Mi4xem0xNyA4My4zaC0zNGwxNy00MC45IDE3IDQwLjl6IiAvPgogIDwvc3ZnPg==",
								],
								[1, "my-test"],
								[1, "component-style-test"],
								[1, "spacer"],
								[
									"aria-label",
									"Angular on twitter",
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://twitter.com/angular",
									"title",
									"Twitter",
								],
								[
									"id",
									"twitter-logo",
									"height",
									"24",
									"data-name",
									"Logo",
									"xmlns",
									"http://www.w3.org/2000/svg",
									"viewBox",
									"0 0 400 400",
								],
								["width", "400", "height", "400", "fill", "none"],
								[
									"d",
									"M153.62,301.59c94.34,0,145.94-78.16,145.94-145.94,0-2.22,0-4.43-.15-6.63A104.36,104.36,0,0,0,325,122.47a102.38,102.38,0,0,1-29.46,8.07,51.47,51.47,0,0,0,22.55-28.37,102.79,102.79,0,0,1-32.57,12.45,51.34,51.34,0,0,0-87.41,46.78A145.62,145.62,0,0,1,92.4,107.81a51.33,51.33,0,0,0,15.88,68.47A50.91,50.91,0,0,1,85,169.86c0,.21,0,.43,0,.65a51.31,51.31,0,0,0,41.15,50.28,51.21,51.21,0,0,1-23.16.88,51.35,51.35,0,0,0,47.92,35.62,102.92,102.92,0,0,1-63.7,22A104.41,104.41,0,0,1,75,278.55a145.21,145.21,0,0,0,78.62,23",
									"fill",
									"#fff",
								],
								[
									"aria-label",
									"Angular on YouTube",
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://youtube.com/angular",
									"title",
									"YouTube",
								],
								[
									"id",
									"youtube-logo",
									"height",
									"24",
									"width",
									"24",
									"data-name",
									"Logo",
									"xmlns",
									"http://www.w3.org/2000/svg",
									"viewBox",
									"0 0 24 24",
									"fill",
									"#fff",
								],
								["d", "M0 0h24v24H0V0z", "fill", "none"],
								[
									"d",
									"M21.58 7.19c-.23-.86-.91-1.54-1.77-1.77C18.25 5 12 5 12 5s-6.25 0-7.81.42c-.86.23-1.54.91-1.77 1.77C2 8.75 2 12 2 12s0 3.25.42 4.81c.23.86.91 1.54 1.77 1.77C5.75 19 12 19 12 19s6.25 0 7.81-.42c.86-.23 1.54-.91 1.77-1.77C22 15.25 22 12 22 12s0-3.25-.42-4.81zM10 15V9l5.2 3-5.2 3z",
								],
								["role", "main", 1, "content"],
								[1, "card", "highlight-card", "card-small"],
								[
									"id",
									"rocket",
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"101.678",
									"height",
									"101.678",
									"viewBox",
									"0 0 101.678 101.678",
								],
								[
									"id",
									"Group_83",
									"data-name",
									"Group 83",
									"transform",
									"translate(-141 -696)",
								],
								[
									"id",
									"Ellipse_8",
									"data-name",
									"Ellipse 8",
									"cx",
									"50.839",
									"cy",
									"50.839",
									"r",
									"50.839",
									"transform",
									"translate(141 696)",
									"fill",
									"#dd0031",
								],
								[
									"id",
									"Group_47",
									"data-name",
									"Group 47",
									"transform",
									"translate(165.185 720.185)",
								],
								[
									"id",
									"Path_33",
									"data-name",
									"Path 33",
									"d",
									"M3.4,42.615a3.084,3.084,0,0,0,3.553,3.553,21.419,21.419,0,0,0,12.215-6.107L9.511,30.4A21.419,21.419,0,0,0,3.4,42.615Z",
									"transform",
									"translate(0.371 3.363)",
									"fill",
									"#fff",
								],
								[
									"id",
									"Path_34",
									"data-name",
									"Path 34",
									"d",
									"M53.3,3.221A3.09,3.09,0,0,0,50.081,0,48.227,48.227,0,0,0,18.322,13.437c-6-1.666-14.991-1.221-18.322,7.218A33.892,33.892,0,0,1,9.439,25.1l-.333.666a3.013,3.013,0,0,0,.555,3.553L23.985,43.641a2.9,2.9,0,0,0,3.553.555l.666-.333A33.892,33.892,0,0,1,32.647,53.3c8.55-3.664,8.884-12.326,7.218-18.322A48.227,48.227,0,0,0,53.3,3.221ZM34.424,9.772a6.439,6.439,0,1,1,9.106,9.106,6.368,6.368,0,0,1-9.106,0A6.467,6.467,0,0,1,34.424,9.772Z",
									"transform",
									"translate(0 0.005)",
									"fill",
									"#fff",
								],
								[
									"id",
									"rocket-smoke",
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"516.119",
									"height",
									"1083.632",
									"viewBox",
									"0 0 516.119 1083.632",
								],
								[
									"id",
									"Path_40",
									"data-name",
									"Path 40",
									"d",
									"M644.6,141S143.02,215.537,147.049,870.207s342.774,201.755,342.774,201.755S404.659,847.213,388.815,762.2c-27.116-145.51-11.551-384.124,271.9-609.1C671.15,139.365,644.6,141,644.6,141Z",
									"transform",
									"translate(-147.025 -140.939)",
									"fill",
									"#f5f5f5",
								],
								[1, "card-container"],
								[
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://angular.io/tutorial",
									1,
									"card",
								],
								[
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"24",
									"height",
									"24",
									"viewBox",
									"0 0 24 24",
									1,
									"material-icons",
								],
								[
									"d",
									"M5 13.18v4L12 21l7-3.82v-4L12 17l-7-3.82zM12 3L1 9l11 6 9-4.91V17h2V9L12 3z",
								],
								["d", "M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"],
								[
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://angular.io/cli",
									1,
									"card",
								],
								[
									"d",
									"M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z",
								],
								[
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://material.angular.io",
									1,
									"card",
								],
								[
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"21.813",
									"height",
									"23.453",
									"viewBox",
									"0 0 179.2 192.7",
									2,
									"margin-right",
									"8px",
								],
								[
									"fill",
									"#ffa726",
									"d",
									"M89.4 0 0 32l13.5 118.4 75.9 42.3 76-42.3L179.2 32 89.4 0z",
								],
								[
									"fill",
									"#fb8c00",
									"d",
									"M89.4 0v192.7l76-42.3L179.2 32 89.4 0z",
								],
								[
									"fill",
									"#ffe0b2",
									"d",
									"m102.9 146.3-63.3-30.5 36.3-22.4 63.7 30.6-36.7 22.3z",
								],
								[
									"fill",
									"#fff3e0",
									"d",
									"M102.9 122.8 39.6 92.2l36.3-22.3 63.7 30.6-36.7 22.3z",
								],
								[
									"fill",
									"#fff",
									"d",
									"M102.9 99.3 39.6 68.7l36.3-22.4 63.7 30.6-36.7 22.4z",
								],
								[
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://blog.angular.io/",
									1,
									"card",
								],
								[
									"d",
									"M13.5.67s.74 2.65.74 4.8c0 2.06-1.35 3.73-3.41 3.73-2.07 0-3.63-1.67-3.63-3.73l.03-.36C5.21 7.51 4 10.62 4 14c0 4.42 3.58 8 8 8s8-3.58 8-8C20 8.61 17.41 3.8 13.5.67zM11.71 19c-1.78 0-3.22-1.4-3.22-3.14 0-1.62 1.05-2.76 2.81-3.12 1.77-.36 3.6-1.21 4.62-2.58.39 1.29.59 2.65.59 4.04 0 2.65-2.15 4.8-4.8 4.8z",
								],
								[
									"target",
									"_blank",
									"rel",
									"noopener",
									"href",
									"https://angular.io/devtools/",
									1,
									"card",
								],
								[
									"xmlns",
									"http://www.w3.org/2000/svg",
									"enable-background",
									"new 0 0 24 24",
									"height",
									"24px",
									"viewBox",
									"0 0 24 24",
									"width",
									"24px",
									"fill",
									"#000000",
									1,
									"material-icons",
								],
								["fill", "none", "height", "24", "width", "24"],
								[
									"d",
									"M14.73,13.31C15.52,12.24,16,10.93,16,9.5C16,5.91,13.09,3,9.5,3S3,5.91,3,9.5C3,13.09,5.91,16,9.5,16 c1.43,0,2.74-0.48,3.81-1.27L19.59,21L21,19.59L14.73,13.31z M9.5,14C7.01,14,5,11.99,5,9.5S7.01,5,9.5,5S14,7.01,14,9.5 S11.99,14,9.5,14z",
								],
								[
									"points",
									"10.29,8.44 9.5,6 8.71,8.44 6.25,8.44 8.26,10.03 7.49,12.5 9.5,10.97 11.51,12.5 10.74,10.03 12.75,8.44",
								],
								["type", "hidden"],
								["selection", ""],
								["tabindex", "0", 1, "card", "card-small", 3, "click"],
								["d", "M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"],
								[1, "terminal", 3, "ngSwitch"],
								[4, "ngSwitchDefault"],
								[4, "ngSwitchCase"],
								[
									"title",
									"Find a Local Meetup",
									"href",
									"https://www.meetup.com/find/?keywords=angular",
									"target",
									"_blank",
									"rel",
									"noopener",
									1,
									"circle-link",
								],
								[
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"24.607",
									"height",
									"23.447",
									"viewBox",
									"0 0 24.607 23.447",
								],
								[
									"id",
									"logo--mSwarm",
									"d",
									"M21.221,14.95A4.393,4.393,0,0,1,17.6,19.281a4.452,4.452,0,0,1-.8.069c-.09,0-.125.035-.154.117a2.939,2.939,0,0,1-2.506,2.091,2.868,2.868,0,0,1-2.248-.624.168.168,0,0,0-.245-.005,3.926,3.926,0,0,1-2.589.741,4.015,4.015,0,0,1-3.7-3.347,2.7,2.7,0,0,1-.043-.38c0-.106-.042-.146-.143-.166a3.524,3.524,0,0,1-1.516-.69A3.623,3.623,0,0,1,2.23,14.557a3.66,3.66,0,0,1,1.077-3.085.138.138,0,0,0,.026-.2,3.348,3.348,0,0,1-.451-1.821,3.46,3.46,0,0,1,2.749-3.28.44.44,0,0,0,.355-.281,5.072,5.072,0,0,1,3.863-3,5.028,5.028,0,0,1,3.555.666.31.31,0,0,0,.271.03A4.5,4.5,0,0,1,18.3,4.7a4.4,4.4,0,0,1,1.334,2.751,3.658,3.658,0,0,1,.022.706.131.131,0,0,0,.1.157,2.432,2.432,0,0,1,1.574,1.645,2.464,2.464,0,0,1-.7,2.616c-.065.064-.051.1-.014.166A4.321,4.321,0,0,1,21.221,14.95ZM13.4,14.607a2.09,2.09,0,0,0,1.409,1.982,4.7,4.7,0,0,0,1.275.221,1.807,1.807,0,0,0,.9-.151.542.542,0,0,0,.321-.545.558.558,0,0,0-.359-.534,1.2,1.2,0,0,0-.254-.078c-.262-.047-.526-.086-.787-.138a.674.674,0,0,1-.617-.75,3.394,3.394,0,0,1,.218-1.109c.217-.658.509-1.286.79-1.918a15.609,15.609,0,0,0,.745-1.86,1.95,1.95,0,0,0,.06-1.073,1.286,1.286,0,0,0-1.051-1.033,1.977,1.977,0,0,0-1.521.2.339.339,0,0,1-.446-.042c-.1-.092-.2-.189-.307-.284a1.214,1.214,0,0,0-1.643-.061,7.563,7.563,0,0,1-.614.512A.588.588,0,0,1,10.883,8c-.215-.115-.437-.215-.659-.316a2.153,2.153,0,0,0-.695-.248A2.091,2.091,0,0,0,7.541,8.562a9.915,9.915,0,0,0-.405.986c-.559,1.545-1.015,3.123-1.487,4.7a1.528,1.528,0,0,0,.634,1.777,1.755,1.755,0,0,0,1.5.211,1.35,1.35,0,0,0,.824-.858c.543-1.281,1.032-2.584,1.55-3.875.142-.355.28-.712.432-1.064a.548.548,0,0,1,.851-.24.622.622,0,0,1,.185.539,2.161,2.161,0,0,1-.181.621c-.337.852-.68,1.7-1.018,2.552a2.564,2.564,0,0,0-.173.528.624.624,0,0,0,.333.71,1.073,1.073,0,0,0,.814.034,1.22,1.22,0,0,0,.657-.655q.758-1.488,1.511-2.978.35-.687.709-1.37a1.073,1.073,0,0,1,.357-.434.43.43,0,0,1,.463-.016.373.373,0,0,1,.153.387.7.7,0,0,1-.057.236c-.065.157-.127.316-.2.469-.42.883-.846,1.763-1.262,2.648A2.463,2.463,0,0,0,13.4,14.607Zm5.888,6.508a1.09,1.09,0,0,0-2.179.006,1.09,1.09,0,0,0,2.179-.006ZM1.028,12.139a1.038,1.038,0,1,0,.01-2.075,1.038,1.038,0,0,0-.01,2.075ZM13.782.528a1.027,1.027,0,1,0-.011,2.055A1.027,1.027,0,0,0,13.782.528ZM22.21,6.95a.882.882,0,0,0-1.763.011A.882.882,0,0,0,22.21,6.95ZM4.153,4.439a.785.785,0,1,0,.787-.78A.766.766,0,0,0,4.153,4.439Zm8.221,18.22a.676.676,0,1,0-.677.666A.671.671,0,0,0,12.374,22.658ZM22.872,12.2a.674.674,0,0,0-.665.665.656.656,0,0,0,.655.643.634.634,0,0,0,.655-.644A.654.654,0,0,0,22.872,12.2ZM7.171-.123A.546.546,0,0,0,6.613.43a.553.553,0,1,0,1.106,0A.539.539,0,0,0,7.171-.123ZM24.119,9.234a.507.507,0,0,0-.493.488.494.494,0,0,0,.494.494.48.48,0,0,0,.487-.483A.491.491,0,0,0,24.119,9.234Zm-19.454,9.7a.5.5,0,0,0-.488-.488.491.491,0,0,0-.487.5.483.483,0,0,0,.491.479A.49.49,0,0,0,4.665,18.936Z",
									"transform",
									"translate(0 0.123)",
									"fill",
									"#f64060",
								],
								[
									"title",
									"Join the Conversation on Discord",
									"href",
									"https://discord.gg/angular",
									"target",
									"_blank",
									"rel",
									"noopener",
									1,
									"circle-link",
								],
								[
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"26",
									"height",
									"26",
									"viewBox",
									"0 0 245 240",
								],
								[
									"d",
									"M104.4 103.9c-5.7 0-10.2 5-10.2 11.1s4.6 11.1 10.2 11.1c5.7 0 10.2-5 10.2-11.1.1-6.1-4.5-11.1-10.2-11.1zM140.9 103.9c-5.7 0-10.2 5-10.2 11.1s4.6 11.1 10.2 11.1c5.7 0 10.2-5 10.2-11.1s-4.5-11.1-10.2-11.1z",
								],
								[
									"d",
									"M189.5 20h-134C44.2 20 35 29.2 35 40.6v135.2c0 11.4 9.2 20.6 20.5 20.6h113.4l-5.3-18.5 12.8 11.9 12.1 11.2 21.5 19V40.6c0-11.4-9.2-20.6-20.5-20.6zm-38.6 130.6s-3.6-4.3-6.6-8.1c13.1-3.7 18.1-11.9 18.1-11.9-4.1 2.7-8 4.6-11.5 5.9-5 2.1-9.8 3.5-14.5 4.3-9.6 1.8-18.4 1.3-25.9-.1-5.7-1.1-10.6-2.7-14.7-4.3-2.3-.9-4.8-2-7.3-3.4-.3-.2-.6-.3-.9-.5-.2-.1-.3-.2-.4-.3-1.8-1-2.8-1.7-2.8-1.7s4.8 8 17.5 11.8c-3 3.8-6.7 8.3-6.7 8.3-22.1-.7-30.5-15.2-30.5-15.2 0-32.2 14.4-58.3 14.4-58.3 14.4-10.8 28.1-10.5 28.1-10.5l1 1.2c-18 5.2-26.3 13.1-26.3 13.1s2.2-1.2 5.9-2.9c10.7-4.7 19.2-6 22.7-6.3.6-.1 1.1-.2 1.7-.2 6.1-.8 13-1 20.2-.2 9.5 1.1 19.7 3.9 30.1 9.6 0 0-7.9-7.5-24.9-12.7l1.4-1.6s13.7-.3 28.1 10.5c0 0 14.4 26.1 14.4 58.3 0 0-8.5 14.5-30.6 15.2z",
								],
								[
									"href",
									"https://github.com/angular/angular",
									"target",
									"_blank",
									"rel",
									"noopener",
								],
								[1, "github-star-badge"],
								["d", "M0 0h24v24H0z", "fill", "none"],
								[
									"d",
									"M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z",
								],
								[
									"d",
									"M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z",
									"fill",
									"#1976d2",
								],
								[
									"id",
									"clouds",
									"xmlns",
									"http://www.w3.org/2000/svg",
									"width",
									"2611.084",
									"height",
									"485.677",
									"viewBox",
									"0 0 2611.084 485.677",
								],
								[
									"id",
									"Path_39",
									"data-name",
									"Path 39",
									"d",
									"M2379.709,863.793c10-93-77-171-168-149-52-114-225-105-264,15-75,3-140,59-152,133-30,2.83-66.725,9.829-93.5,26.25-26.771-16.421-63.5-23.42-93.5-26.25-12-74-77-130-152-133-39-120-212-129-264-15-54.084-13.075-106.753,9.173-138.488,48.9-31.734-39.726-84.4-61.974-138.487-48.9-52-114-225-105-264,15a162.027,162.027,0,0,0-103.147,43.044c-30.633-45.365-87.1-72.091-145.206-58.044-52-114-225-105-264,15-75,3-140,59-152,133-53,5-127,23-130,83-2,42,35,72,70,86,49,20,106,18,157,5a165.625,165.625,0,0,0,120,0c47,94,178,113,251,33,61.112,8.015,113.854-5.72,150.492-29.764a165.62,165.62,0,0,0,110.861-3.236c47,94,178,113,251,33,31.385,4.116,60.563,2.495,86.487-3.311,25.924,5.806,55.1,7.427,86.488,3.311,73,80,204,61,251-33a165.625,165.625,0,0,0,120,0c51,13,108,15,157-5a147.188,147.188,0,0,0,33.5-18.694,147.217,147.217,0,0,0,33.5,18.694c49,20,106,18,157,5a165.625,165.625,0,0,0,120,0c47,94,178,113,251,33C2446.709,1093.793,2554.709,922.793,2379.709,863.793Z",
									"transform",
									"translate(142.69 -634.312)",
									"fill",
									"#eee",
								],
							],
							template: function (e, t) {
								if (1 & e) {
									let e = r.ɵɵgetCurrentView();
									r.ɵɵelementStart(0, "div", 0),
										r.ɵɵelement(1, "img", 1),
										r.ɵɵelementStart(2, "span"),
										r.ɵɵtext(3, "Welcome"),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(4, "span", 2),
										r.ɵɵtext(5, " This text should be red (global style)"),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(6, "span", 3),
										r.ɵɵtext(7, " This text should be blue (component style)"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(8, "div", 4),
										r.ɵɵelementStart(9, "a", 5),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(10, "svg", 6),
										r.ɵɵelement(11, "rect", 7)(12, "path", 8),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(13, "a", 9),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(14, "svg", 10),
										r.ɵɵelement(15, "path", 11)(16, "path", 12),
										r.ɵɵelementEnd()()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(17, "div", 13)(18, "div", 14),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(19, "svg", 15)(20, "title"),
										r.ɵɵtext(21, "Rocket Ship"),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(22, "g", 16),
										r.ɵɵelement(23, "circle", 17),
										r.ɵɵelementStart(24, "g", 18),
										r.ɵɵelement(25, "path", 19)(26, "path", 20),
										r.ɵɵelementEnd()()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(27, "span"),
										r.ɵɵtext(28),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(29, "svg", 21)(30, "title"),
										r.ɵɵtext(31, "Rocket Ship Smoke"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(32, "path", 22),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(33, "h2"),
										r.ɵɵtext(34, "Resources"),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(35, "p"),
										r.ɵɵtext(
											36,
											"Here are some links to help you get started:",
										),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(37, "div", 23)(38, "a", 24),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(39, "svg", 25),
										r.ɵɵelement(40, "path", 26),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(41, "span"),
										r.ɵɵtext(42, "Learn Angular"),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(43, "svg", 25),
										r.ɵɵelement(44, "path", 27),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(45, "a", 28),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(46, "svg", 25),
										r.ɵɵelement(47, "path", 29),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(48, "span"),
										r.ɵɵtext(49, "CLI Documentation"),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(50, "svg", 25),
										r.ɵɵelement(51, "path", 27),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(52, "a", 30),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(53, "svg", 31),
										r.ɵɵelement(54, "path", 32)(55, "path", 33)(56, "path", 34)(
											57,
											"path",
											35,
										)(58, "path", 36),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(59, "span"),
										r.ɵɵtext(60, "Angular Material"),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(61, "svg", 25),
										r.ɵɵelement(62, "path", 27),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(63, "a", 37),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(64, "svg", 25),
										r.ɵɵelement(65, "path", 38),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(66, "span"),
										r.ɵɵtext(67, "Angular Blog"),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(68, "svg", 25),
										r.ɵɵelement(69, "path", 27),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(70, "a", 39),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(71, "svg", 40)(72, "g"),
										r.ɵɵelement(73, "rect", 41),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(74, "g")(75, "g"),
										r.ɵɵelement(76, "path", 42)(77, "polygon", 43),
										r.ɵɵelementEnd()()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(78, "span"),
										r.ɵɵtext(79, "Angular DevTools"),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(80, "svg", 25),
										r.ɵɵelement(81, "path", 27),
										r.ɵɵelementEnd()()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(82, "h2"),
										r.ɵɵtext(83, "Next Steps"),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(84, "p"),
										r.ɵɵtext(85, "What do you want to do next with your app?"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(86, "input", 44, 45),
										r.ɵɵelementStart(88, "div", 23)(89, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "component"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(90, "svg", 25),
										r.ɵɵelement(91, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(92, "span"),
										r.ɵɵtext(93, "New Component"),
										r.ɵɵelementEnd()(),
										r.ɵɵelementStart(94, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "material"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(95, "svg", 25),
										r.ɵɵelement(96, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(97, "span"),
										r.ɵɵtext(98, "Angular Material"),
										r.ɵɵelementEnd()(),
										r.ɵɵelementStart(99, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "pwa"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(100, "svg", 25),
										r.ɵɵelement(101, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(102, "span"),
										r.ɵɵtext(103, "Add PWA Support"),
										r.ɵɵelementEnd()(),
										r.ɵɵelementStart(104, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "dependency"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(105, "svg", 25),
										r.ɵɵelement(106, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(107, "span"),
										r.ɵɵtext(108, "Add Dependency"),
										r.ɵɵelementEnd()(),
										r.ɵɵelementStart(109, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "test"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(110, "svg", 25),
										r.ɵɵelement(111, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(112, "span"),
										r.ɵɵtext(113, "Run and Watch Tests"),
										r.ɵɵelementEnd()(),
										r.ɵɵelementStart(114, "button", 46),
										r.ɵɵlistener("click", function () {
											r.ɵɵrestoreView(e);
											let t = r.ɵɵreference(87);
											return r.ɵɵresetView((t.value = "build"));
										}),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(115, "svg", 25),
										r.ɵɵelement(116, "path", 47),
										r.ɵɵelementEnd(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(117, "span"),
										r.ɵɵtext(118, "Build for Production"),
										r.ɵɵelementEnd()()(),
										r.ɵɵelementStart(119, "div", 48),
										r.ɵɵtemplate(120, s, 2, 0, "pre", 49),
										r.ɵɵtemplate(121, l, 2, 0, "pre", 50),
										r.ɵɵtemplate(122, a, 2, 0, "pre", 50),
										r.ɵɵtemplate(123, u, 2, 0, "pre", 50),
										r.ɵɵtemplate(124, d, 2, 0, "pre", 50),
										r.ɵɵtemplate(125, c, 2, 0, "pre", 50),
										r.ɵɵelementEnd(),
										r.ɵɵelementStart(126, "div", 23)(127, "a", 51),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(128, "svg", 52)(129, "title"),
										r.ɵɵtext(130, "Meetup Logo"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(131, "path", 53),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(132, "a", 54),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(133, "svg", 55)(134, "title"),
										r.ɵɵtext(135, "Discord Logo"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(136, "path", 56)(137, "path", 57),
										r.ɵɵelementEnd()()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(138, "footer"),
										r.ɵɵtext(139, " Love Angular?\xa0 "),
										r.ɵɵelementStart(140, "a", 58),
										r.ɵɵtext(141, " Give our repo a star. "),
										r.ɵɵelementStart(142, "div", 59),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(143, "svg", 25),
										r.ɵɵelement(144, "path", 60)(145, "path", 61),
										r.ɵɵelementEnd(),
										r.ɵɵtext(146, " Star "),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelementStart(147, "a", 58),
										r.ɵɵnamespaceSVG(),
										r.ɵɵelementStart(148, "svg", 25),
										r.ɵɵelement(149, "path", 62)(150, "path", 60),
										r.ɵɵelementEnd()()(),
										r.ɵɵelementStart(151, "svg", 63)(152, "title"),
										r.ɵɵtext(153, "Gray Clouds Background"),
										r.ɵɵelementEnd(),
										r.ɵɵelement(154, "path", 64),
										r.ɵɵelementEnd()(),
										r.ɵɵnamespaceHTML(),
										r.ɵɵelement(155, "router-outlet");
								}
								if (2 & e) {
									let e = r.ɵɵreference(87);
									r.ɵɵadvance(28),
										r.ɵɵtextInterpolate1("", t.title, " app is running!"),
										r.ɵɵadvance(91),
										r.ɵɵproperty("ngSwitch", e.value),
										r.ɵɵadvance(2),
										r.ɵɵproperty("ngSwitchCase", "material"),
										r.ɵɵadvance(1),
										r.ɵɵproperty("ngSwitchCase", "pwa"),
										r.ɵɵadvance(1),
										r.ɵɵproperty("ngSwitchCase", "dependency"),
										r.ɵɵadvance(1),
										r.ɵɵproperty("ngSwitchCase", "test"),
										r.ɵɵadvance(1),
										r.ɵɵproperty("ngSwitchCase", "build");
								}
							},
							dependencies: [
								o.NgSwitch,
								o.NgSwitchCase,
								o.NgSwitchDefault,
								i.RouterOutlet,
							],
							styles: [
								'[_nghost-%COMP%]{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif,"Apple Color Emoji","Segoe UI Emoji","Segoe UI Symbol";font-size:14px;color:#333;box-sizing:border-box;-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale}h1[_ngcontent-%COMP%], h2[_ngcontent-%COMP%], h3[_ngcontent-%COMP%], h4[_ngcontent-%COMP%], h5[_ngcontent-%COMP%], h6[_ngcontent-%COMP%]{margin:8px 0}p[_ngcontent-%COMP%]{margin:0}.spacer[_ngcontent-%COMP%]{flex:1}.toolbar[_ngcontent-%COMP%]{position:absolute;top:0;left:0;right:0;height:60px;display:flex;align-items:center;background-color:#1976d2;color:#fff;font-weight:600}.toolbar[_ngcontent-%COMP%]   img[_ngcontent-%COMP%]{margin:0 16px}.toolbar[_ngcontent-%COMP%]   #twitter-logo[_ngcontent-%COMP%]{height:40px;margin:0 8px}.toolbar[_ngcontent-%COMP%]   #youtube-logo[_ngcontent-%COMP%]{height:40px;margin:0 16px}.toolbar[_ngcontent-%COMP%]   #twitter-logo[_ngcontent-%COMP%]:hover, .toolbar[_ngcontent-%COMP%]   #youtube-logo[_ngcontent-%COMP%]:hover{opacity:.8}.content[_ngcontent-%COMP%]{display:flex;margin:82px auto 32px;padding:0 16px;max-width:960px;flex-direction:column;align-items:center}svg.material-icons[_ngcontent-%COMP%]{height:24px;width:auto}svg.material-icons[_ngcontent-%COMP%]:not(:last-child){margin-right:8px}.card[_ngcontent-%COMP%]   svg.material-icons[_ngcontent-%COMP%]   path[_ngcontent-%COMP%]{fill:#888}.card-container[_ngcontent-%COMP%]{display:flex;flex-wrap:wrap;justify-content:center;margin-top:16px}.card[_ngcontent-%COMP%]{all:unset;border-radius:4px;border:1px solid #eee;background-color:#fafafa;height:40px;width:200px;margin:0 8px 16px;padding:16px;display:flex;flex-direction:row;justify-content:center;align-items:center;transition:all .2s ease-in-out;line-height:24px}.card-container[_ngcontent-%COMP%]   .card[_ngcontent-%COMP%]:not(:last-child){margin-right:0}.card.card-small[_ngcontent-%COMP%]{height:16px;width:168px}.card-container[_ngcontent-%COMP%]   .card[_ngcontent-%COMP%]:not(.highlight-card){cursor:pointer}.card-container[_ngcontent-%COMP%]   .card[_ngcontent-%COMP%]:not(.highlight-card):hover{transform:translateY(-3px);box-shadow:0 4px 17px rgba(0,0,0,.35)}.card-container[_ngcontent-%COMP%]   .card[_ngcontent-%COMP%]:not(.highlight-card):hover   .material-icons[_ngcontent-%COMP%]   path[_ngcontent-%COMP%]{fill:#696767}.card.highlight-card[_ngcontent-%COMP%]{background-color:#1976d2;color:#fff;font-weight:600;border:none;width:auto;min-width:30%;position:relative}.card.card.highlight-card[_ngcontent-%COMP%]   span[_ngcontent-%COMP%]{margin-left:60px}svg#rocket[_ngcontent-%COMP%]{width:80px;position:absolute;left:-10px;top:-24px}svg#rocket-smoke[_ngcontent-%COMP%]{height:calc(100vh - 95px);position:absolute;top:10px;right:180px;z-index:-10}a[_ngcontent-%COMP%], a[_ngcontent-%COMP%]:visited, a[_ngcontent-%COMP%]:hover{color:#1976d2;text-decoration:none}a[_ngcontent-%COMP%]:hover{color:#125699}.terminal[_ngcontent-%COMP%]{position:relative;width:80%;max-width:600px;border-radius:6px;padding-top:45px;margin-top:8px;overflow:hidden;background-color:#0f0f10}.terminal[_ngcontent-%COMP%]::before{content:"•••";position:absolute;top:0;left:0;height:4px;background:#3a3a3a;color:#c2c3c4;width:100%;font-size:2rem;line-height:0;padding:14px 0;text-indent:4px}.terminal[_ngcontent-%COMP%]   pre[_ngcontent-%COMP%]{font-family:SFMono-Regular,Consolas,Liberation Mono,Menlo,monospace;color:#fff;padding:0 1rem 1rem;margin:0}.circle-link[_ngcontent-%COMP%]{height:40px;width:40px;border-radius:40px;margin:8px;background-color:#fff;border:1px solid #eee;display:flex;justify-content:center;align-items:center;cursor:pointer;box-shadow:0 1px 3px rgba(0,0,0,.12),0 1px 2px rgba(0,0,0,.24);transition:1s ease-out}.circle-link[_ngcontent-%COMP%]:hover{transform:translateY(-0.25rem);box-shadow:0px 3px 15px rgba(0,0,0,.2)}footer[_ngcontent-%COMP%]{margin-top:8px;display:flex;align-items:center;line-height:20px}footer[_ngcontent-%COMP%]   a[_ngcontent-%COMP%]{display:flex;align-items:center}.github-star-badge[_ngcontent-%COMP%]{color:#24292e;display:flex;align-items:center;font-size:12px;padding:3px 10px;border:1px solid rgba(27,31,35,.2);border-radius:3px;background-image:linear-gradient(-180deg, #fafbfc, #eff3f6 90%);margin-left:4px;font-weight:600}.github-star-badge[_ngcontent-%COMP%]:hover{background-image:linear-gradient(-180deg, #f0f3f6, #e6ebf1 90%);border-color:rgba(27,31,35,.35);background-position:-0.5em}.github-star-badge[_ngcontent-%COMP%]   .material-icons[_ngcontent-%COMP%]{height:16px;width:16px;margin-right:4px}svg#clouds[_ngcontent-%COMP%]{position:fixed;bottom:-160px;left:-230px;z-index:-10;width:1920px}@media screen and (max-width: 767px){.card-container[_ngcontent-%COMP%] > *[_ngcontent-%COMP%]:not(.circle-link), .terminal[_ngcontent-%COMP%]{width:100%}.card[_ngcontent-%COMP%]:not(.highlight-card){height:16px;margin:8px 0}.card.highlight-card[_ngcontent-%COMP%]   span[_ngcontent-%COMP%]{margin-left:72px}svg#rocket-smoke[_ngcontent-%COMP%]{right:120px;transform:rotate(-5deg)}}@media screen and (max-width: 575px){svg#rocket-smoke[_ngcontent-%COMP%]{display:none;visibility:hidden}}.component-style-test[_ngcontent-%COMP%]{color:blue}',
							],
						})),
						e
					);
				})();
			},
			"./src/app/app.module.ts": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 }),
					Object.defineProperty(t, "AppModule", {
						enumerable: !0,
						get: function () {
							return l;
						},
					});
				var r = n(
						"../../node_modules/@angular/platform-browser/fesm2022/platform-browser.mjs",
					),
					o = n("./src/app/app-routing.module.ts"),
					i = n("./src/app/app.component.ts"),
					s = n.ir(n("../../node_modules/@angular/core/fesm2022/core.mjs"));
				let l = (() => {
					class e {}
					return (
						(e.ɵfac = function (t) {
							return new (t || e)();
						}),
						(e.ɵmod = s.ɵɵdefineNgModule({
							type: e,
							bootstrap: [i.AppComponent],
						})),
						(e.ɵinj = s.ɵɵdefineInjector({
							imports: [r.BrowserModule, o.AppRoutingModule],
						})),
						e
					);
				})();
			},
			"./src/main.ts": function (e, t, n) {
				"use strict";
				Object.defineProperty(t, "__esModule", { value: !0 });
				var r = n.ir(
						n(
							"../../node_modules/@angular/platform-browser/fesm2022/platform-browser.mjs",
						),
					),
					o = n("./src/app/app.module.ts");
				r.platformBrowser()
					.bootstrapModule(o.AppModule)
					.catch((e) => console.error(e));
			},
		},
		t = {};
	function n(r) {
		var o = t[r];
		if (void 0 !== o) return o.exports;
		var i = (t[r] = { exports: {} });
		return e[r](i, i.exports, n), i.exports;
	}
	!(function () {
		function e(t) {
			if ("function" != typeof WeakMap) return null;
			var n = new WeakMap(),
				r = new WeakMap();
			return (e = function (e) {
				return e ? r : n;
			})(t);
		}
		n.ir = function (t, n) {
			if (!n && t && t.__esModule) return t;
			if (null === t || ("object" != typeof t && "function" != typeof t))
				return { default: t };
			var r = e(n);
			if (r && r.has(t)) return r.get(t);
			var o = {},
				i = Object.defineProperty && Object.getOwnPropertyDescriptor;
			for (var s in t)
				if ("default" !== s && Object.prototype.hasOwnProperty.call(t, s)) {
					var l = i ? Object.getOwnPropertyDescriptor(t, s) : null;
					l && (l.get || l.set)
						? Object.defineProperty(o, s, l)
						: (o[s] = t[s]);
				}
			return (o.default = t), r && r.set(t, o), o;
		};
	})(),
		(n.es = function (e, t) {
			return (
				Object.keys(e).forEach(function (n) {
					"default" !== n &&
						!Object.prototype.hasOwnProperty.call(t, n) &&
						Object.defineProperty(t, n, {
							enumerable: !0,
							get: function () {
								return e[n];
							},
						});
				}),
				e
			);
		}),
		n("./src/main.ts");
})();
