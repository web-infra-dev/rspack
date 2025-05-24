async function testCircularDependenciesA() {
	console.log("Testing circular dependencies (a)...");
	try {
		const { default: value, b, ba } = await import("container/a");
		console.log("container/a imported:", { value, b, ba });

		if (value === "a" && b === "b" && ba === "a") {
			console.log("✅ Test A passed: circular dependencies work correctly");
		} else {
			console.log("❌ Test A failed:", {
				expected: { value: "a", b: "b", ba: "a" },
				actual: { value, b, ba }
			});
		}
	} catch (error) {
		console.log("❌ Test A failed with error:", error);
	}
}

async function testCircularDependenciesB() {
	console.log("Testing circular dependencies (b)...");
	try {
		const { default: value, a, ab } = await import("container2/b");
		console.log("container2/b imported:", { value, a, ab });

		if (value === "b" && a === "a" && ab === "b") {
			console.log("✅ Test B passed: circular dependencies work correctly");
		} else {
			console.log("❌ Test B failed:", {
				expected: { value: "b", a: "a", ab: "b" },
				actual: { value, a, ab }
			});
		}
	} catch (error) {
		console.log("❌ Test B failed with error:", error);
	}
}

// Run the tests
testCircularDependenciesA();
testCircularDependenciesB();
