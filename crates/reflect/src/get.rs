#[macro_export]
macro_rules! get {
    // Entry: start with an immutable borrow of the root expression
    ($root:expr => $($path:tt)+) => {
        $crate::get!(@acc &($root) ; $($path)+)
    };

    // The path must start with a slash: /seg/seg/...
    (@acc $acc:expr ; / $($rest:tt)+ ) => {
        $crate::get!(@seg $acc ; $($rest)+)
    };

    // --- identifier segment (struct/tuple-struct field) ---
    // More segments after this one
    (@seg $acc:expr ; $field:ident / $($rest:tt)+ ) => {
        $crate::get!(@acc ::core::ops::Index::index($acc, stringify!($field)) ; / $($rest)+ )
    };
    // Terminal identifier segment
    (@seg $acc:expr ; $field:ident ) => {
        ::core::ops::Index::index($acc, stringify!($field))
    };

    // --- literal segment (numbers, strings, etc.) using Index ---
    // More segments after this one
    (@seg $acc:expr ; $idx:literal / $($rest:tt)+ ) => {
        $crate::get!(@acc ::core::ops::Index::index($acc, $idx) ; / $($rest)+ )
    };
    // Terminal literal segment
    (@seg $acc:expr ; $idx:literal ) => {
        ::core::ops::Index::index($acc, $idx)
    };
}

#[cfg(test)]
mod test {
    use crate::value_of;

    #[test]
    pub fn basic() {
        let meta = crate::MetaData::from([("a", value_of!(21)), ("b", value_of!(true))]);

        let out = get!(meta => /a);
        assert_eq!(out, &value_of!(21));
    }

    #[test]
    pub fn dynamic_sequence_index() {
        let arr: [i32; 3] = [3, 2, 1];
        let value = value_of!(arr);

        let seq = value.as_dynamic().as_sequence();
        assert_eq!(seq.index(1).to_i32(), 2);
    }
}
