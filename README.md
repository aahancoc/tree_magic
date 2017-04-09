# TreeMagic

TreeMagic is a Rust library that determines the file type a given file or byte stream. Right now only Linux systems are supported. 

Unlike the typical approach that libmagic and file(1) uses, this loads all the file types in a graph. Then, instead of checking the file agains *every* file type, it can traverse down the tree and only check the file types that make sense to check. (After all, the fastest code is the code that never gets run.)

This library also provides the ability to check if a file is a certain type without going through the pains of checking it against every file type.

For the time being, all mime information and relation information is loaded from the Shared MIME-info Database as described at https://specifications.freedesktop.org/shared-mime-info-spec/shared-mime-info-spec-latest.html.

It is planned to have custom file checking functions for many types. For instance, everything that subclasses `application/zip` can be determined further by peeking at the zip's file structure. Files like scripts and program sources can be checked against a quick and dirty parser instead of the weird herusitics used now.

Hopefully this will be quicker and more accurate than the standard libmagic approach. It's already quicker, actually. From a completely unscientific test of my Downloads folder, TreeMagic takes 0.138s while file-5.30 with -i flag needs 0.884s. Still kind of sucky on accuracy, though, but that's because this uses a different magic file and it's missing a lot of entries.
