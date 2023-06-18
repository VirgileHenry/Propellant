
/// Compute a compile-time hash of the given string. 
/// This allows to name object by string by the programmer, yet still being as efficient as ints.
pub const fn id(input: &str) -> u64 {
    const_fnv1a_hash::fnv1a_hash_str_64(input)
}

/// Compute a compile-time hash of the given string. 
/// This allows to name object by string by the programmer, yet still being as efficient as ints.
pub const fn small_id(input: &str) -> u32 {
    const_fnv1a_hash::fnv1a_hash_str_32(input)
}