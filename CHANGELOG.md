# Changelog
All notable changes to this project will be documented in this file.

## [0.55.2] - 2018-04-20
### Changed
- Make configurations and pipeline objects public
- Bump snips-nlu-ontology to `0.54.3`

### Fixed
- Bug with prefix and suffix features

## [0.55.1] - 2018-04-10
### Added
- Add support for the `length` feature function in slot filling feature extractrion

### Changed
- Bump ontology from `0.54.1` to `0.54.2`

## [0.55.0] - 2018-04-05
### Added
- Add ability to create an NLU engine directly from a file

### Fixed
- Fix issue with builtin entities during slot filling

### Changed
- Bump model version from `0.13.0` to `0.14.0`
- Improve intent classification by leveraging builtin entities
- Improve support for japanese
- Rename python package to `snips_nlu_rust`


[0.55.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.55.1...0.55.2
[0.55.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.55.0...0.55.1
[0.55.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.54.0...0.55.0