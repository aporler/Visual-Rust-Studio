# Visual Rust Studio — explanations of Rust diagnostics (English, United States)
#
# Keys: rustc-<error code> (e.g. rustc-E0308) and lint-<lint name>.
# Each explanation is linked to an official code: it never replaces the original
# compiler message, which stays visible.

rustc-E0061 = The function is called with the wrong number of arguments. Compare the call with the function signature (parameters between parentheses).
rustc-E0106 = A reference (&) is missing a lifetime. When a struct or a function returns a reference, Rust needs to know how long the referenced data lives. Often, returning an owned value (String instead of &str) is simpler.
rustc-E0277 = A type does not implement a required trait. For example, you cannot print with {"{}"} a type that does not implement Display, or add values of different types. Check the types involved or implement/derive the trait (#[derive(Debug)], etc.).
rustc-E0308 = Mismatched types: Rust expected one type but found another. For example, a String where a number is expected. Convert the value (parse(), to_string(), as) or change the declared type.
rustc-E0369 = The operator (+, -, ==, …) cannot be used with these types. Both operands must be compatible, or the type must implement the corresponding trait (Add, PartialEq…).
rustc-E0382 = Ownership: the value was moved (given) to another variable or function, so it can no longer be used here. Use a reference (&value), clone it (.clone()) or reorganize the code.
rustc-E0384 = A variable declared without mut cannot be modified. Write let mut name = … if the value must change.
rustc-E0412 = This type is unknown at this location. Check its spelling, or import it with use (for example use std::collections::HashMap;).
rustc-E0425 = This name (variable or function) is unknown here. Check its spelling and make sure it is declared before use and in the same scope (between the same braces).
rustc-E0432 = The import (use) cannot be resolved. Check the module path and that the dependency is declared in Cargo.toml.
rustc-E0433 = Rust cannot resolve this path: a module or crate is probably missing an import (use) or a dependency in Cargo.toml.
rustc-E0499 = Borrowing: you cannot have two mutable references (&mut) to the same value at the same time. Finish using the first one before creating the second, or split the data.
rustc-E0502 = Borrowing: the value is already borrowed immutably (&) while you try to modify it (&mut), or vice versa. Shorten the lifetime of one of the borrows, for example by copying the needed value first.
rustc-E0505 = You are trying to move a value while it is still borrowed. Finish using the reference first, or clone the value.
rustc-E0596 = You are trying to modify through a non-mutable binding or reference. Add mut to the variable (let mut) or take a mutable reference (&mut).
rustc-E0597 = The value does not live long enough: it is destroyed at the end of its block while a reference to it is still used. Move the declaration to an outer block or return an owned value.
rustc-E0599 = This method does not exist for this type, or the trait providing it is not imported. Check the type of the value and the spelling of the method.
lint-unused_variables = This variable is never used. Remove it, or prefix its name with _ to show that this is intentional.
lint-unused_imports = This import (use) is not used. You can remove it.
lint-unused_mut = This variable is declared mut but is never modified. You can remove mut.
lint-dead_code = This code (function, field, type) is never used.
