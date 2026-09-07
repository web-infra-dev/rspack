function sameOwner(requestAccount, currentAccount) {
	return requestAccount === currentAccount;
}

function resumeCreateOrder(submitAccount, currentAccount) {
	$stateMachine: for (;;) {
		switch (0) {
			case 0: {
				const comparator = sameOwner;
				let requestAccount;

				$checkNotNull: do {
					if (submitAccount == null) {
						throw new Error("Required value was null.");
					}

					requestAccount = submitAccount;
					break $checkNotNull;
				} while (false);

				if (comparator(requestAccount, currentAccount)) {
					return "same";
				}

				return "changed";
			}

			default:
				continue $stateMachine;
		}
	}
}

it("should preserve labeled do-while control flow during minification", () => {
	expect(resumeCreateOrder("account-a", "account-b")).toBe("changed");
});
