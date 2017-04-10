# tree_magic

tree_magic is a Rust library that determines the file type a given file or byte stream. 

Read the documentation at https://docs.rs/tree_magic/

Unlike the typical approach that libmagic and file(1) uses, this loads all the file types in a graph based on subclasses. (EX: `application/vnd.openxmlformats-officedocument.wordprocessingml.document` (MS Office 2007) subclasses `application/zip` which subclasses `application/octet-stream`) Then, instead of checking the file against *every* file type, it can traverse down the tree and only check the file types that make sense to check. (After all, the fastest check is the check that never gets run.)

This library also provides the ability to check if a file is a certain type without going through the process of checking it against every file type.

A simple command-line client `tmagic` is also provided that acts as a replacement for `file -i`, excluding charset information.

## Performance

Hopefully this will be quicker and more accurate than the standard libmagic approach. It's already significantly quicker, actually. From a completely unscientific test of my Downloads folder, `tree_magic` takes 0.138s while file-5.30 with -i flag needs 0.884s.

However, it should be noted that the FreeDesktop.org magic files cover less filetypes than the magic files used by libmagic. (It is, however, significantly easier to parse, as it only covers magic numbers and not attributes or anything like that.) See the TODO section for plans to fix this.

## Compatibility

Right now only Linux systems (or anything else that supports FreeDesktop.org standards) are fully supported. Other systems will only report "inode/directory", "text/plain", or "application/octet-stream". This is expected to be improved in the future.

All mime information and relation information is loaded from the Shared MIME-info Database as described at https://specifications.freedesktop.org/shared-mime-info-spec/shared-mime-info-spec-latest.html.

### Architecture

`tree_magic` is split up into different "checker" modules. Each checker handles a certain set of filetypes, and only those. For instance, the `basetype` checker handles the `inode/*` and `text/plain` types, while the `fdo_magic` checker handles anything with a magic number.

The point is, instead of following the `libmagic` route of having one magic descriptor format that fits every file, we can specialize and choose the checker that suits the file format best.

During library initialization, each checker is queried for the types is supports and the parent->child relations between them. During this time, the checkers can load any rules, schemas, etc. into memory. **Time during the initialization phase is less valuable than time in the checking phase**, as the library only gets initialized once, and the library can check thousands of files during a program's lifetime.

From the list of file types and relations, a directed graph is built, and each node is added to a hash map. The caller can use these directly to find parents, children, etc. of a given MIME if needed.

When a file needs to be checked against a certain MIME, each checker is queried to see if it supports that type, and if so, it runs the checker. If it returns true, it must be that type.

When a file needs it's MIME type found, the library starts at the `all/all` node of the type graph (or whichever node the user specifies) and walks down the tree. If a match is found, it continues searching down that branch. If no match is found, it retrieves the deepest MIME type found.

## TODO

### Improve fdo-magic checker

Right now the `fdo-magic` checker does not handle endianess. It also does not handle magic files stored in the user's home directory.

### Additional checkers

It is planned to have custom file checking functions for many types. Here's some ideas:

- `zip`: Everything that subclasses `application/zip` can be determined further by peeking at the zip's directory listing. 

- `grep`: Text files such as program scripts and configuration files could be parsed with a regex (or whatever works best). 

- `json`, `toml`, etc: Check the given file against a schema and return true if it matches. (By this point there should be few enough potential matches that it should be okay to load the entire file)

- (specialized parsers): Binary (or text) files without any sort of magic can be checked for compliance against a quick and dirty `nom` parser instead of the weird heuristics used by libmagic.

To add additional checker types, add a new module exporting:

- `init::get_supported() -> Vec<(String)>`

- `init::get_subclasses() -> Vec<String, String)>`

- `test::can_check(&str) -> bool`

- `test::from_u8(&[u8], &str) -> bool`

- `test::from_filpath(&str, &str) -> Result<bool, std::io::Error>`
    
and then add calls to those functions in `graph_init`, `match_u8` and `match_filepath` in `lib.rs`.

### Caching

It would be really nice for a checker (like `basetype` or that json/toml example) to be able to cache an in-memory representation of the file, so it doesn't have to get re-loaded and re-parsed for every new type. I think the best way would be to have `from_*` pass around something mutable with a trait `MimeCache` (or whatever), but I could easily be wrong.

It would also be nice if we could cache which checkers support which files. We could probably do this in a `lazy_static!` HashMap.

### Multiple file types

There are some weird files out there ( [Polyglot quines](https://en.wikipedia.org/wiki/Polyglot_(computing)) come to mind. ) that are multiple file types. This might be worth handling for security reasons. (It's not a huge priority, though.)
