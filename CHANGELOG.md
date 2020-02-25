# 0.2.3

Upgraded package versions to latest (except nom, which is currently stuck at
3.x) and fixed the paths in the doc tests 

# 0.2.2

Yanked due to accidental breaking API change

# 0.2.1

Incorporated fix by Bram Sanders to prevent panic on non-existent file.

# 0.2.0

Major changes, front-end and back.

- Added `is_alias` function
- `from_*` functions excluding `from_*_node` now return MIME, not Option<MIME>
- New feature flag: `staticmime`. Changes type of MIME from String to &'static str
- Bundled magic file, so it works on Windows as well.
- Split `fdo_magic` checker into `fdo_magic::sys` and `fdo_magic::builtin`
- `len` argument removed from `*_u8` functions
- Tests and benchmarks added.
- Fixed horribly broken logic in `fdo_magic` checker
- Checks the most common types before obscure types
- Changed hasher to `fnv`.
- Added support for handling aliases in input
- `tmagic` command has more features
- Major speed improvements

# 0.1.1
 
- *Changed public interface*: Added `from_u8` export function
- *Changed public interface*: Changed len argument for `u8` functions from `u32` to `usize`
- Minor speed improvements in `fdo_magic` checker
 
# 0.1.0
 
Initial release
