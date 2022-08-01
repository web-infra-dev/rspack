**The style guide should always be WIP. We will modify this document with the project growing**

# Reference:  

1. https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/style.md 
2. https://github.com/pingcap/style-guide/tree/master/docs/rust 

# Convention in CodeStyle 

1. Each rule has two levels: **Error** and **Warn**  
  a. Error. If you don't follow the rule, the reviewer will request a change 
  b. Warn. If you don't follow the rule, the reviewer may suggest a change(we follow the 
community convention Nit: It would be better to do xxxxx), but you could 
ignore it. A pull request is welcome to refactor it. 
# General 

## Crates.io Dependencies 

We try to be very conservative with the usage of crates.io dependencies. 

Don't use small "helper" crates (exception: itertools and either are allowed). 

If there's some general reusable bit of code you need, consider adding it to the common utils 

crate. 

A useful exercise is to read Cargo.lock  and see if some transitive dependencies do not make 

sense for rspack. 

**Rationale:**  keep compile times low, create ecosystem pressure for faster compiles, reduce the 

number of things which might break. 

## Clippy 

We don't enforce Clippy. 

A number of default lints have a high false positive rate.

Selectively patching false-positives with allow(clippy) is considered worse than not using 

Clippy at all. 

Careful tweaking of lint is welcome. 


Of course, applying Clippy suggestions is welcome as long as they indeed improve the code. 

**Rationale:**  see rust-lang/clippy#5537. 

# Code 

## Marked Tests 

## #[should_panic]  (Warn)  

Do not use #[should_panic] tests. 

Instead, explicitly check for None, Err, etc. 

**Rationale:**  #[should_panic] is a tool for library authors to make sure that the API does not 

fail silently when misused. 

rspack is not a library, we don't need to test for API misuse, and we have to handle any user 

input without panic as much possible as we can. 

Panic messages in the logs from the #[should_panic] tests are confusing. 

## #[ignore]  (Warn)  

Do not #[ignore] tests. 

If the test currently does not work, assert the wrong behavior and add a fixme explaining why it 

is wrong. 

**Rationale:**  noticing when the behavior is fixed, making sure that even the wrong behavior is 

acceptable (ie, not a panic). 

## Function Preconditions  (Warn)  

Express function preconditions in types and force the caller to provide them (rather than 

checking in callee): 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
```
```
// GOOD
fn frobnicate(walrus: Walrus) {
...
}
```
```
// BAD
fn frobnicate(walrus: Option<Walrus>) {
let walrus = match walrus {
Some(it) => it,
None => return,
};
...
```

```
13 }
```
**Rationale:**  this makes control flow explicit at the call site. 

Call-site has more context, it often happens that the precondition falls out naturally or can be 

bubbled up higher in the stack. 

Avoid splitting precondition check and precondition use across functions: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
20
21
22
23
24
25
26
27
```
```
// GOOD
fn main() {
let s: &str = ...;
if let Some(contents) = string_literal_contents(s) {
```
```
}
}
```
```
fn string_literal_contents(s: &str) -> Option<&str> {
if s.starts_with('"') && s.ends_with('"') {
Some(&s[ 1 ..s.len() - 1 ])
} else {
None
}
}
```
```
// BAD
fn main() {
let s: &str = ...;
if is_string_literal(s) {
let contents = &s[ 1 ..s.len() - 1 ];
}
}
```
```
fn is_string_literal(s: &str) -> bool {
s.starts_with('"') && s.ends_with('"')
}
```
In the "Not as good" version, the precondition that  1  is a valid char boundary is checked in 

is_string_literal and used in foo. 

In the "Good" version, the precondition check  and usage are checked in the same block, and 

then encoded in the types. 

**Rationale:**  non-local code properties degrade under change. 


## Control Flow  (Warn)  

As a special case of the previous rule, do not hide control flow inside functions, push it to the 

caller: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
```
```
// GOOD
if cond {
f()
}
```
```
// BAD
fn f() {
if !cond {
return;
}
...
}
```
## Getters & Setters  (Warn)  

If a field can have any value without breaking invariants, make the field public. 

Conversely, if there is an invariant, document it, enforce it in the "constructor" function, make 

the field private, and provide a getter. 

Getters should return borrowed data:  ** (Error in most cases, unless we need to clone in some **

**special case.)**  

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
```
```
struct Person {
// Invariant: never empty
first_name: String,
middle_name: Option<String>
}
```
```
// GOOD
impl Person {
fn first_name(&self) -> &str { self.first_name.as_str() }
fn middle_name(&self) -> Option<&str> { self.middle_name.as_ref() }
}
```
```
// BAD
impl Person {
fn first_name(&self) -> String { self.first_name.clone() }
fn middle_name(&self) -> &Option<String> { &self.middle_name }
}
```

**Rationale:**  Non-local code properties degrade under change, privacy makes invariant local. 

Borrowed owned types (&String) disclose irrelevant details about internal representation. 

Irrelevant (neither right nor wrong) things obscure correctness. 

## Use Generic Types  (Error)  

More generally, always prefer types on the left 

```
1
2
3
4
5
```
```
// GOOD BAD
&[T] &Vec<T>
&str &String
Option<&T> &Option<T>
&Path &PathBuf
```
**Rationale:**  types on the left are strictly more general. 

Even when generality is not required, consistency is important. 

## Constructors  (Error)  

Prefer Default to zero-argument new function. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
```
```
// GOOD
#[derive(Default)]
struct Foo {
bar: Option<Bar>
}
```
```
// BAD
struct Foo {
bar: Option<Bar>
}
```
```
impl Foo {
fn new() -> Foo {
Foo { bar: None }
}
}
```
Prefer Default even if it has to be implemented manually. 

**Rationale:**  less typing in the common case, uniformity. 


Avoid using "dummy" states to implement a Default. 

If a type doesn't have a sensible default, empty value, don't hide it. 

Let the caller explicitly decide what the right initial state is. 

## Functions Over Objects  (Error)  

Avoid creating "doer" objects. 

That is, objects which are created only to execute a single action. 

```
1
2
3
4
5
```
```
// GOOD
do_thing(arg1, arg2);
```
```
// BAD
ThingDoer::new(arg1, arg2).do();
```
Note that this concerns only outward API. 

When implementing do_thing, it might be very useful to create a context object. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
```
```
pub fn do_thing(arg1: Arg1, arg2: Arg2) -> Res {
let mut ctx = Ctx { arg1, arg2 };
ctx.run()
}
```
```
struct Ctx {
arg1: Arg1, arg2: Arg
}
```
```
impl Ctx {
fn run(self) -> Res {
...
}
}
```
The difference is that Ctx is an impl detail here. 

Sometimes a middle ground is acceptable if this can save some busywork: 

```
1
2
3
4
5
```
```
ThingDoer::do(arg1, arg2);
```
```
pub struct ThingDoer {
arg1: Arg1, arg2: Arg2,
}
```

```
6
7
8
9
10
11
12
13
14
```
```
impl ThingDoer {
pub fn do(arg1: Arg1, arg2: Arg2) -> Res {
ThingDoer { arg1, arg2 }.run()
}
fn run(self) -> Res {
...
}
}
```
**Rationale:**  not bothering the caller with irrelevant details, not mixing user API with implementor 

API. 

## Functions with many parameters  (Error)  

Avoid creating functions with many optional or boolean parameters. 

Introduce a Config struct instead. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
20
21
22
23
24
25
```
```
// GOOD
pub struct AnnotationConfig {
pub binary_target: bool,
pub annotate_runnables: bool,
pub annotate_impls: bool,
}
```
```
pub fn annotations(
db: &RootDatabase,
file_id: FileId,
config: AnnotationConfig
) -> Vec<Annotation> {
...
}
```
```
// BAD
pub fn annotations(
db: &RootDatabase,
file_id: FileId,
binary_target: bool,
annotate_runnables: bool,
annotate_impls: bool,
) -> Vec<Annotation> {
...
}
```

**Rationale:**  reducing large codebase refactoring due to a simple config change. 

If the function has many parameters, they most likely change frequently. 

By packing them into a struct we protect all intermediary functions from changes. 

Do not implement Default for the Config struct, the caller has more context to determine 

better defaults.  **(Warn)**  

Do not store Config as a part of the state, pass it explicitly.  **(Warn)**  

This gives more flexibility for the caller. 

If there is variation not only in the input parameters, but in the return type as well, consider 

introducing a Command type. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
```
```
// MAYBE GOOD
pub struct Query {
pub name: String,
pub case_sensitive: bool,
}
```
```
impl Query {
pub fn all(self) -> Vec<Item> { ... }
pub fn first(self) -> Option<Item> { ... }
}
```
```
// MAYBE BAD
fn query_all(name: String, case_sensitive: bool) -> Vec<Item> { ... }
fn query_first(name: String, case_sensitive: bool) -> Option<Item> { ... }
```
## Prefer Separate Functions Over Parameters  (Error)  

If a function has a bool or an Option parameter, and it is always called with true, 

false, Some and None literals, split the function in two. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
```
```
// GOOD
fn caller_a() {
foo()
}
```
```
fn caller_b() {
foo_with_bar(Bar::new())
}
```
```
fn foo() { ... }
fn foo_with_bar(bar: Bar) { ... }
```

```
13
14
15
16
17
18
19
20
21
22
```
```
// BAD
fn caller_a() {
foo(None)
}
```
```
fn caller_b() {
foo(Some(Bar::new()))
}
```
```
fn foo(bar: Option<Bar>) { ... }
```
**Rationale:**  more often than not, such functions display "false sharing" -- they have 

additional if branching inside for two different cases. 

Splitting the two different control flows into two functions simplifies each path, and remove 

cross-dependencies between the two paths. 

If there's common code between foo and foo_with_bar, extract that into a common 

helper. 

# Performance 

## Avoid Allocations  (Error)  

Avoid writing code which is slower than it needs to be. 

Don't allocate a Vec where an iterator would do, don't allocate strings needlessly. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
```
```
// GOOD
use itertools::Itertools;
```
```
let (first_word, second_word) = match text.split_ascii_whitespace().collect_tuple
Some(it) => it,
None => return,
}
```
```
// BAD
let words = text.split_ascii_whitespace().collect::<Vec<_>>();
if words.len() != 2 {
return
}
```
**Rationale:**  not allocating is almost always faster. 

## Push Allocations to the Call Site  (Error)  


If an allocation is inevitable, let the caller allocate the resource: 

```
1 2 3 4 5 6 7 8 9
```
```
10
```
```
// GOOD
fn frobnicate(s: String) {
...
}
```
```
// BAD
fn frobnicate(s: &str) {
let s = s.to_string();
...
}
```
**Rationale:**  reveals the costs. 

-  It is also more efficient when the caller already owns the allocation. 

- Avoiding implicit clone.  See discussion https://github.com/speedy-js/rspack/pull/454 

## Collection Types  (Error)  

Prefer rustc_hash::FxHashMap and rustc_hash::FxHashSet or any other faster hash 

library instead of the ones in std::collections. 

**Rationale:**  they use a hasher that's significantly faster and using them consistently will reduce 

code size by some small amount. 

## Avoid Intermediate Collections  (Error)  

When writing a recursive function to compute a sets of things, use an accumulator parameter 

instead of returning a fresh collection. 

Accumulator goes first in the list of arguments. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
```
```
// GOOD
pub fn reachable_nodes(node: Node) -> FxHashSet<Node> {
let mut res = FxHashSet::default();
go(&mut res, node);
res
}
fn go(acc: &mut FxHashSet<Node>, node: Node) {
acc.insert(node);
for n in node.neighbors() {
go(acc, n);
}
}
```

```
14
15
16
17
18
19
20
21
22
```
```
// BAD
pub fn reachable_nodes(node: Node) -> FxHashSet<Node> {
let mut res = FxHashSet::default();
res.insert(node);
for n in node.neighbors() {
res.extend(reachable_nodes(n));
}
res
}
```
**Rationale:**  re-use allocations, accumulator style is more concise and performant for complex 

cases. 

## Avoiding the use of concurrent data structure in single thread 

## scenario  (Error)  

### Example 

2. https://github.com/speedy-js/rspack/pull/211#discussion_r876826072 

**Rationale:**  Single-threaded data structure would always be better than concurrent data 

structure(less memory consumption and lock free) in no concurrent or parallel scenario 

## Prefer to write simple, clear code unless optimization is necessary. 

## Unless you can optimize it by hand.  (Error)  

### Example: 

3. https://github.com/speedy-js/rspack/pull/219#discussion_r882379045 

```
1
2
3
4
```
```
// BAD
if id.starts_with("globals:") {
let global_id = id.replace("globals:", "");
}
```
```
1
2
3
4
```
```
// Good
if id.starts_with("globals:") {
let global_id = &id[8..].to_string();
}
```

```
The str::replace has a time complexity of O(n) and will reallocate a string the length of 
n. 
As a comparison, &id[8..].to_string() could reduce the O(n) time complexity for 
searching. 
Also, it is easy to optimize. 
```
**Rationale: See the Miscellaneous below**  

## Miscellaneous  (Error)  

```
Rationale:  Rust uses monomorphization to compile generic code, meaning that for each 
instantiation of a generic functions with concrete types, the function is compiled afresh, per 
crate. 
This allows for exceptionally good performance, but leads to increased compile times. 
Runtime performance obeys 80%/20% rule -- only a small fraction of code is hot.
Compile time does not obey this rule -- all code has to be compiled.
```
# •

"Premature optimization is the root of all evil" 

## ◦Optimization should be justified by profiles and benchmarks. 

## ◦Prefer to write simple, clear code unless optimization is necessary. 

# •

Don't write sloppy code: 

## ◦Use iterators. 

## ◦Don't use Arc or Mutex for single-threaded code (use Rc or RefCell instead). 

## ◦Avoid global, mutable state. 

## ◦Prefer to use push and push_str to build strings in performance-sensitive code, 

rather than the format macro. 

## ◦Don't worry about empty Vecs/Strings - they don't allocate and are very cheap. 

## ◦Consider the computational complexity of algorithms, but bear in mind the expected size 

of input. 

# • Don't hide potentially expensive code 

## ◦code that may do IO, block, or sleep should be clearly named and documented. 

## ◦allocation should be either expected from the purpose of a function or be documented 

(e.g., conversion functions should not allocate). 

```
The concurrency of your code is likely to have a large impact on its performance. 
Consider if and how code can be concurrent at the design phase. 
Design to avoid locking or other forms of synchronization if code is performance-sensitive. 
Rationale:  
```

```
Modern compilers and CPUs can optimize simple code more effectively than complicated 
code; modern CPUs are complex, it is very difficult to predict whether hand-optimised code 
is effective. 
```

Allocation is a primary cause of slowdown in many programs. 


Use iterators to avoid slow bounds checks. 


```
Performant code requires scaling to multiple threads, blocking threads on locks, IO, etc., kills 
parallelism. 
```
## Style 

## Order of Imports  (warn)  

```
Separate import groups with blank  lines. 
Use one use per crate. 
Module declarations come before the imports. 
Order them in "suggested reading order" for a person new to the code base. 
```
```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
```
```
mod x;
mod y;
```
```
// First std.
use std::{ ... }
```
```
// Second, external crates (both crates.io crates and other rspack crates).
use crate_foo::{ ... }
use crate_bar::{ ... }
```
```
// Then current crate.
use crate::{}
```
```
// Finally, parent and child modules, but prefer `use crate::`.
use super::{}
```
```
// Re-exports are treated as item definitions rather than imports, so they go
// after imports and modules. Use them sparingly.
pub use crate::x::Z;
```
```
Rationale:  consistency. 
Reading order is important for new contributors. 
Grouping by crate allows spotting unwanted dependencies easier. 
```

## Import Style  (Warn)  

When implementing traits from std::fmt or std::ops, import the module: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
```
```
// GOOD
use std::fmt;
```
```
impl fmt::Display for RenameError {
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { .. }
}
```
```
// BAD
impl std::fmt::Display for RenameError {
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { .. }
}
```
```
// BAD
use std::ops::Deref;
```
```
impl Deref for Widget {
type Target = str;
fn deref(&self) -> &str { .. }
}
```
**Rationale:**  overall, less typing. 

Makes it clear that a trait is implemented, rather than used. 

Avoid local use MyEnum::* imports. 

**Rationale:**  consistency. 

Prefer use crate::foo::bar to use super::bar or use self::bar::baz. 

**Rationale:**  consistency, this is the style which works in all cases. 

By default, avoid re-exports. 

**Rationale:**  for non-library code, re-exports introduce two ways to use something and allow for 

inconsistency. 

**Exception** : We need to unify version of swc related crate in rspack , like 

## Order of Items  (Warn)  

Optimize for the reader who sees the file for the first time, and wants to get a general idea about 

what's going on. 

People read things from top to bottom, so place most important things first. 

Specifically, if all items except one are private, always put the non-private item on top. 


```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
20
21
22
23
24
25
26
27
```
```
// GOOD
pub(crate) fn frobnicate() {
Helper::act()
}
```
```
#[derive(Default)]
struct Helper { stuff: i32 }
```
```
impl Helper {
fn act(&self) {
```
```
}
}
```
```
// BAD
#[derive(Default)]
struct Helper { stuff: i32 }
```
```
pub(crate) fn frobnicate() {
Helper::act()
}
```
```
impl Helper {
fn act(&self) {
```
```
}
}
```
If there's a mixture of private and public items, put public items first. 

Put structs and enums first, functions and impls last. Order type declarations in top-down 

manner. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
```
```
// GOOD
struct Parent {
children: Vec<Child>
}
```
```
struct Child;
```
```
impl Parent {
}
```
```
impl Child {
```

```
12
13
14
15
16
17
18
19
20
21
22
23
24
25
```
```
}
```
```
// BAD
struct Child;
```
```
impl Child {
}
```
```
struct Parent {
children: Vec<Child>
}
```
```
impl Parent {
}
```
**Rationale:**  easier to get the sense of the API by visually scanning the file. 

If function bodies are folded in the editor, the source code should read as documentation for the 

public API. 

## Context Parameters  (Warn)  

Some parameters are threaded unchanged through many function calls. 

They determine the "context" of the operation. 

Pass such parameters first, not last. 

If there are several context parameters, consider packing them into a struct Ctx and 

passing it as &self. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
```
```
// GOOD
fn dfs(graph: &Graph, v: Vertex) -> usize {
let mut visited = FxHashSet::default();
return go(graph, &mut visited, v);
```
```
fn go(graph: &Graph, visited: &mut FxHashSet<Vertex>, v: usize) -> usize {
...
}
}
```
```
// BAD
fn dfs(v: Vertex, graph: &Graph) -> usize {
fn go(v: usize, graph: &Graph, visited: &mut FxHashSet<Vertex>) -> usize {
...
}
```
```
let mut visited = FxHashSet::default();
```

```
18
19
```
```
go(v, graph, &mut visited)
}
```
```
Rationale:  consistency. 
Context-first works better when non-context parameter is a lambda. 
```
# Naming  

## Variable N aming  (Warn)  

```
The default name is a lower_cased name of the type: global_state: GlobalState. 
Default names: 
```
# •

res, ret -- "result of the function" local variable 

# • n_foos -- number of foos (prefer this to foo_count) 

# •

foo_idx -- index of foo 

# • ctx -- Context 

**Rationale:**  consistency. 

## Convention  (Error)  

**Prefer meaningful names to short names.**  

# •

Types and traits: UpperCamelCase, 

# •

Enum variants: UpperCamelCase, 

## ◦Do not prefix with the enum name, e.g., Some not OptionSome, 

# • Struct fields: snake_case, 

# •

Function and method names: snake_case, 

# • Local variables: snake_case, 

# •

Macro names: snake_case, 

# • Constants (consts and immutable statics): SCREAMING_SNAKE_CASE, 

# •

```
Crate names: kebab-case in Cargo and snake_case in Rust code (but prefer single 
word names where possible), 
```
# • Module names: snake_case. 

All names should be from English. 

```
When a name is forbidden because it is a reserved word (e.g., type, crate), use an 
abbreviation (e.g., ty) or an underscore suffix (e.g., crate_). 
Use raw identifiers (e.g., r#type, r#crate) if necessary; e.g., you are working with 
generated code or code which interacts with data using identifier names (e.g., JSON or protobuf 
```

libraries). 

Prefer using full words rather than abbreviations, e.g., diagnostic and 

expansion_config rather than diag and expn_cfg. 

It's ok to use abbreviations where they are well-known and standard outside the codebase (e.g., 

in the Rust or database communities). 

E.g., ctx for 'context', cf for 'column family', or expr for expression. 

When in doubt, use the full name. 

Use acronyms where the acronym is standard, e.g., Sql rather than 

StandardQueryLanguage. 

Treat acronyms as words, e.g., GrpcType or grpc_variable, not GRPCType or 

g_r_p_c_variable/GRPC_variable. 

Treat contractions of multiple words as one word, e.g., Stdin rather than StdIn. 

Where it is clear from the wider context, if is fine to use short (even single character) names for 

local variables with very narrow scope. 

Some examples: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
```
```
// Argument to inline closure (`s`).
vec.iter().map(|s| s.len());
// Single line `match` arm (`e`).
match ... {
Ok(_) => { ... }
Err(e) => println!("Error: {}", e),
}
// Counter in `for` loop (`i`).
for (i, label) in labels.iter().enumerate() {
...
}
```
When interfacing with non-Rust code, prefer to use Rust conventions in the Rust code, rather 

than conventions from another language. 

This may mean that items in Rust and non-Rust code have different names, but the correlation 

should be clear. 

Where necessary, use aliases or wrappers to avoid using unconventional names for FFI items. 

Generic type and lifetime parameters should usually have short (usually single letter) names, 

e.g., T and 'a. 

Use longer names for generics if there are multiple generics in scope and short names are 

confusing. 

Associated types should have descriptive names like any other type. 

## Method n ames  (Error)  


A primary constructor function should be called new. 

A new constructor may take values for all fields or provide some defaults. 

Secondary constructors (which usually customise more values than in new) should usually be 

prefixed with with_, e.g., with_capacity. 

A conversion method which preserves the original and gives a reference to a different type 

should be called as_type where type is the type being converted into. 

E.g., as_str, not as_str_ref or get_str or str. 

Such a conversion should be very cheap, usually just a different view on the original. 

A conversion method which consumes the original and gives an owned value should be called 

into_type. 

E.g., into_string. 

An into_type method should not clone any part of the original (unless it is very cheap, e.g., 

Copy types). 

An expensive conversion method (e.g., one which clones the original) should be called 

to_type. 

E.g., to_string or to_str. 

Where a smart pointer or other wrapper type has a method which consumes the wrapper and 

returns the inner type, the method should be called into_inner if it will never panic and 

unwrap if it might panic. 

Where the wrapper is not consumed and a reference is returned, use get. 

For other unwrapping methods, follow the naming conventions used by Box, Option, and 

Result. 

The default method for iterating over the contents of a type should be called iter. 

A method for iteration which consumes the collection should be called into_iter, however, 

such a method should usually be in an implementation of IntoIterator. 

Prefer not to use getter and setter methods, but if you do they should be called foo and 

set_foo (where foo is the name of the field), not get_foo. Method for checking the 

presence of a field should have an is_ or has_ prefix and return a bool. 

When naming traits: "Prefer (transitive) verbs, nouns, and then adjectives; avoid grammatical 

suffixes (like -able)". 

E.g., Copy, Send, Encode. 

Variations of methods which have different ownership or mutability properties from the default, 

should use suffixes to distinguish the variations. 

E.g., foo for the default variation (whether that is owned or by-ref), foo_ref for a by-

reference version, and foo_mut for a by-mutable-reference version. 

Don't use multiple suffixes, e.g. foo_mut_ref. 

In most cases, mutability does not need to be reflected in method names. 


```
Where it is required to disambiguate between mutable and non-mutable return types, for 
example, use mut should be a prefix, not a suffix. 
E.g., as_mut_slice, not as_slice_mut. 
```
```
Rationale  
Existing guidance: 
```
# •

Rust style guide 

# •

Rust API design guidelines 

# •

RFC 199 

# •

RFC 344 

# •

RFC 430 

## Early Returns  (Warn)  

Do use early returns 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
```
```
// GOOD
fn foo() -> Option<Bar> {
if !condition() {
return None;
}
```
```
Some(...)
}
```
```
// BAD
fn foo() -> Option<Bar> {
if condition() {
Some(...)
} else {
None
}
}
```
**Rationale:**  reduce cognitive stack usage. 

Use return Err(err) to throw an error: 

```
1
2
3
4
```
```
// GOOD
fn f() -> Result<(), ()> {
if condition {
return Err(());
```

```
5
6
7
8
9
10
11
12
13
14
15
```
```
}
Ok(())
}
```
```
// BAD
fn f() -> Result<(), ()> {
if condition {
Err(())?;
}
Ok(())
}
```
**Rationale:**  return has type !, which allows the compiler to flag dead 

code (Err(...)? is of unconstrained generic type T). 

## If-let  (Error)  

Avoid if let ... { } else { } construct, use match instead. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
```
```
// GOOD
match ctx.expected_type.as_ref() {
Some(expected_type) => completion_ty == expected_type && !expected_type.is_un
None => false,
}
```
```
// BAD
if let Some(expected_type) = ctx.expected_type.as_ref() {
completion_ty == expected_type && !expected_type.is_unit()
} else {
false
}
```
**Rationale:**  match is almost always more compact. 

The else branch can get a more precise pattern: None or Err(_) instead of _. 

## Match Ergonomics  (Warn)  

Don't use the ref keyword. 

**Rationale:**  consistency & simplicity. 

ref was required before match ergonomics. 

Today, it is redundant. 

Between ref and mach ergonomics, the latter is more ergonomic in most cases, and is simpler 

(does not require a keyword). 


##### Empty Match Arms  (Warn)  

Use => (), when a match arm is intentionally empty: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
```
```
// GOOD
match result {
Ok(_) => (),
Err(err) => error!("{}", err),
}
```
```
// BAD
match result {
Ok(_) => {}
Err(err) => error!("{}", err),
}
```
**Rationale:**  consistency. 

## Functional Combinator  (Warn)  

Use high order monadic combinators like map, then when they are a natural choice; don't 

bend the code to fit into some combinator. 

If writing a chain of combinators creates friction, replace them with control flow constructs: 

for, if, match. 

Mostly avoid bool::then and Option::filter. 

```
1 2 3 4 5 6 7 8
// GOOD
if !x.cond() {
return None;
}
Some(x)
```
```
// BAD
Some(x).filter(|it| it.cond())
```
This rule is more "soft" then others, and boils down mostly to taste. 

The guiding principle behind this rule is that code should be dense in computation, and sparse 

in the number of expressions per line. 

The second example contains less computation -- the filter function is an indirection for 

if, it doesn't do any useful work by itself. 

At the same time, it is more crowded -- it takes more time to visually scan it. 


**Rationale:**  consistency, playing to language's strengths. 

Rust has first-class support for imperative control flow constructs like for and if, while 

functions are less first-class due to lack of universal function type, currying, and non-first-class 

effects (?, .await). 

## Using f unction c ombinator p roperly c ould m ake y our c ode m ore c oncise 

Example: 

1. https://github.com/speedy-js/rspack/pull/427#discussion_r913858458 

## Turbofish  (Warn)  

Prefer type ascription over the turbofish. 

When ascribing types, avoid _ 

```
1 2 3 4 5 6 7 8
// GOOD
let mutable: Vec<T> = old.into_iter().map(|it| builder.make_mut(it)).collect();
```
```
// BAD
let mutable: Vec<_> = old.into_iter().map(|it| builder.make_mut(it)).collect();
```
```
// BAD
let mutable = old.into_iter().map(|it| builder.make_mut(it)).collect::<Vec<_>>();
```
**Rationale:**  consistency, readability. 

If compiler struggles to infer the type, the human would as well. 

Having the result type specified up-front helps with understanding what the chain of iterator 

methods is doing. 

## Helper Functions (Warn) 

Avoid creating single-use helper functions: 

```
1 2 3 4 5 6 7 8
// GOOD
let buf = {
let mut buf = get_empty_buf(&mut arena);
buf.add_item(item);
buf
};
```
```
// BAD
```

```
9
10
11
12
13
14
15
16
17
```
```
let buf = prepare_buf(&mut arena, item);
```
```
...
```
```
fn prepare_buf(arena: &mut Arena, item: Item) -> ItemBuf {
let mut res = get_empty_buf(&mut arena);
res.add_item(item);
res
}
```
Exception: if you want to make use of return or ?. 

**Rationale:**  single-use functions change frequently, adding or removing parameters adds churn. 

A block serves just as well to delineate a bit of logic, but has access to all the context. 

Re-using originally single-purpose function often leads to bad coupling. 

## Local Helper Functions  (Warn)  

Put nested helper functions at the end of the enclosing functions 

(this requires using return statement). 

Don't nest more than one level deep. 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
17
18
19
```
```
// GOOD
fn dfs(graph: &Graph, v: Vertex) -> usize {
let mut visited = FxHashSet::default();
return go(graph, &mut visited, v);
```
```
fn go(graph: &Graph, visited: &mut FxHashSet<Vertex>, v: usize) -> usize {
...
}
}
```
```
// BAD
fn dfs(graph: &Graph, v: Vertex) -> usize {
fn go(graph: &Graph, visited: &mut FxHashSet<Vertex>, v: usize) -> usize {
...
}
```
```
let mut visited = FxHashSet::default();
go(graph, &mut visited, v)
}
```
**Rationale:**  consistency, improved top-down readability. 


## Helper Variables  (Warn)

Introduce helper variables freely, especially for multiline conditions: 

```
1 2 3 4 5 6 7 8 9
```
```
10
11
12
13
14
15
16
```
```
// GOOD
let rustfmt_not_installed =
captured_stderr.contains("not installed") || captured_stderr.contains("not av
```
```
match output.status.code() {
Some( 1 ) if !rustfmt_not_installed => Ok(None),
_ => Err(format_err!("rustfmt failed:\n{}", captured_stderr)),
};
```
```
// BAD
match output.status.code() {
Some( 1 )
if !captured_stderr.contains("not installed")
&& !captured_stderr.contains("not available") => Ok(None),
_ => Err(format_err!("rustfmt failed:\n{}", captured_stderr)),
};
```
**Rationale:**  Like blocks, single-use variables are a cognitively cheap abstraction, as they have 

access to all the context. 

Extra variables help during debugging, they make it easy to print/view  important intermediate 

results. 

Giving a name to a condition inside an if expression often improves clarity and leads to nicely 

formatted code. 

## Documentation (Since we write doc comments rarely, Warn or Off ?) 

Style inline code comments as proper sentences. 

Start with a capital letter, end with a dot. 

```
1 2 3 4 5 6 7 8 9
```
```
10
```
```
// GOOD
```
```
// Only simple single segment paths are allowed.
MergeBehavior::Last => {
tree.use_tree_list().is_none() && tree.path().map(path_len) <= Some( 1 )
}
```
```
// BAD
```
```
// only simple single segment paths are allowed
```

```
11
12
13
```
```
MergeBehavior::Last => {
tree.use_tree_list().is_none() && tree.path().map(path_len) <= Some( 1 )
}
```
**Rationale:**  writing a sentence (or maybe even a paragraph) rather just "a comment" creates a 

more appropriate frame of mind. 

It tricks you into writing down more of the context you keep in your head while coding. 

For .md and .adoc files, prefer a sentence-per-line format, don't wrap lines. 

If the line is too long, you want to split the sentence in two :-) 

**Rationale:**  much easier to edit the text and read the diff, see this link. 


