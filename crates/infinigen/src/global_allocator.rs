#[cfg(all(
    feature = "jemalloc",
    not(target_env = "msvc"),
    not(target_family = "wasm")
))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(
    feature = "jemalloc",
    not(target_env = "msvc"),
    not(target_family = "wasm")
))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub fn check_global_allocator() {
    #[cfg(all(feature = "jemalloc", target_env = "msvc"))]
    panic!("Jemalloc feature does not work on Windows");
    #[cfg(all(feature = "jemalloc", target_family = "wasm"))]
    panic!("Jemalloc feature does not work on WASM");
}
