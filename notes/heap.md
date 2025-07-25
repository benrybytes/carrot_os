
# static variables

are not mutable by default, unless encapsulated by a mutex to prevent data races

rather than manually allocating and deallocating, we want to use RAII (Resource Acquisition is initialization)
where, the pointer referencing a value can automatically clean up after usage (like unique_ptr and shared_ptr in c++)

in rust, it would be the Box::new to serve as a unique_ptr

# GlobalAlloc

is a trait that allows realloc and alloc methods

it is an unsafe trait due to thinking the programmer implements the trait correctly

methods are unsafe as it thinks the invariants (different parameter and trait usage) are correct
