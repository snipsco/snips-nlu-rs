# Changelog
All notable changes to this project will be documented in this file.

## [0.65.1] - 2019-09-04
### Added
- Add new `parse_with_alternatives` API to `SnipsNluEngine` to get alternative intents and slot values [#148](https://github.com/snipsco/snips-nlu-rs/pull/148)

## [0.65.0] - 2019-07-11
### Changed
- Bumped ontology to `0.65.0` [#143](https://github.com/snipsco/snips-nlu-rs/pull/143)
- Bumped parsers to `0.3.0` [#143](https://github.com/snipsco/snips-nlu-rs/pull/143)

### Added
- Added a lookup intent parser [#121](https://github.com/snipsco/snips-nlu-rs/pull/121)
- Introduce new builtin entities:
    - `snips/time`
    - `snips/timePeriod`
    - `snips/date`
    - `snips/datePeriod`
    - `snips/city`
    - `snips/country`
    - `snips/region`

## [0.64.4] - 2019-06-19
### Changed
- Accept ambiguous results from `DeterministicIntentParser` when confidence score is above 0.5 [#141](https://github.com/snipsco/snips-nlu-rs/pull/141)
- Bumped ontology to `0.64.7` [#142](https://github.com/snipsco/snips-nlu-rs/pull/142)
- Bumped parsers to `0.2.2` [#142](https://github.com/snipsco/snips-nlu-rs/pull/142)

## [0.64.3] - 2019-04-29
### Fixed
- Make the `WrongModelVersion` error message intelligible [#133](https://github.com/snipsco/snips-nlu-rs/pull/133)
- Fix error handling in Python wrapper [#134](https://github.com/snipsco/snips-nlu-rs/pull/134)
- Return an error when using unknown intents in whitelist or blacklist [#136](https://github.com/snipsco/snips-nlu-rs/pull/136)
- Fix issue with stop words in `DeterministicIntentParser` [#137](https://github.com/snipsco/snips-nlu-rs/pull/137)
- Fix caching issue in `CustomEntityParser` [#138](https://github.com/snipsco/snips-nlu-rs/pull/138)

### Changed
- Re-score ambiguous `DeterministicIntentParser` results based on slots [#139](https://github.com/snipsco/snips-nlu-rs/pull/139)

## [0.64.2] - 2019-04-09
### Fixed
- Fix handling of ambiguous utterances in `DeterministicIntentParser` [#129](https://github.com/snipsco/snips-nlu-rs/pull/129)
- Stop normalizing confidence scores when there is an intents filter [#130](https://github.com/snipsco/snips-nlu-rs/pull/130)

### Added
- Add new APIs in ffi and bindings (python, kotlin, swift) [#131](https://github.com/snipsco/snips-nlu-rs/pull/131)

### Changed
- Leverage entity scopes of each intent in deterministic intent parser [#126](https://github.com/snipsco/snips-nlu-rs/pull/126)

## [0.64.1] - 2019-03-01
### Fixed
- Fix bug with regex patterns containing duplicated slot names [#124](https://github.com/snipsco/snips-nlu-rs/pull/124)

## [0.64.0] - 2019-02-28
### Changed
- Bumped `snips-nlu-ontology` to 0.64.4
- Bumped `crf-suite` to 0.3.1 and cbindgen to

## [0.63.1] - 2019-02-11
### Fixed
- Fix an issue regarding the way builtin entities were handled by the `CRFSlotFiller` [#116](https://github.com/snipsco/snips-nlu-rs/pull/116)

## [0.63.0] - 2019-02-04
### Added
- `get_intents` API: get the probabilities of all intents (including the null intent) with respect to an input text
- Pass `--top_intents` to the parsing CLI to use the `get_intents` API instead of `parse`
- `get_slots` API: extract slots by providing a text along with its corresponding intent
- Added a an optional `CooccurrenceVectorizer` to the `Featurizer` that extracts co-occurrence features

### Changed
- A probability is now returned when no intent is found
- The `parse` API now takes a new optional parameter `intents_blacklist` which allows to filter out specific intents
- `Slot` object now contains an optional `confidence_score` attribute
- `intent` value of `IntentParserResult` is no longer optional: the optionality is moved to `intent_name` in the `IntentClassificationResult` object
- `slots` value of `IntentParserResult` is no longer optional (`None` is replaced by empty `Vec`)
- Update to Rust 2018
- Refactored the `Featurizer` and moved its attributes to an underlying `TfidfVectorizer`

## [0.61.2] - 2019-01-17
### Changed
- Bump `snips-nlu-ontology` to `0.61.3`

## [0.61.1] - 2018-12-14
### Changed
- Bump `snips-nlu-ontology` to `0.61.2`

### Fixed
- Issue when resolving custom entities

## [0.62.0] - 2018-11-26
### Changed
- Bumped `snips-nlu-ontology` to `0.62.0`

### Fixed
- Made CI faster by running the full test suite only when merging on `master`

### Added
- Added a script to update `snips-nlu-ontology` everywhere in the codebase
- Added SNIPS_NLU_VERSION in `libsnips_nlu.h` and a cbindgen.toml to help us generating `libsnips_nlu.h` automatically
- Added logs in the build script

## [0.61.0] - 2018-10-16
### Changed
- Entity injection API is now handled by an `NLUInjector` object

### Added
- Support for builtin music entities in english

### Fixed
- Handle stemming properly in entity injection 

## [0.60.1] - 2018-10-09
### Added
- Entity injection API for both custom entities and builtin gazetteer entities

### Fixed
- Swift wrapper
- `DeterministicIntentParser` now relies on the custom entity parser

## [0.60.0] - 2018-10-05
### Added
- Support for 3 new builtin entities in French: `snips/musicAlbum`, `snips/musicArtist` and `snips/musicTrack`

### Changed
- model version `0.16.0` => `0.17.0`
- Replace `snips-nlu-cli` crate with Rust example

### Fixed
- Bug with entity feature name in intent classification

## [0.59.0] - 2018-09-26
### Added
- Limited support for Italian by bumping the `snips-nlu-ontology` to `0.58.0` and `snips-nlu-utils` to `0.7.0`

### Changed
- Stopped creating a useless `CRFSlotFiller` in the `ProbabilisticIntentParser` when the intent has no slot

## [0.58.3] - 2018-08-23
### Fixed
- Fix mapping issue when multiple synonyms have same normalization

## [0.58.2] - 2018-08-21
### Changed
- Bump `snips-nlu-ontology` to `0.57.3`

## [0.58.1] - 2018-07-24
### Fixed
- Error when loading a `SnipsNluEngine` from zip data

## [0.58.0] - 2018-07-17
### Added
- Interactive parsing CLI

### Changed
- The `SnipsNluEngine` object is now loaded from a directory instead of a single json file 
(see https://github.com/snipsco/snips-nlu/releases/tag/0.16.0).
- Language resources are now loaded *dynamically* from the trained engine directory instead of 
being statically hardcoded, reducing the binary size by 31Mb.

### Removed
- `snips-nlu-resources` and `snips-nlu-resources-packed` crates no longer exists.
- `FileBasedConfiguration`, `ZipBasedConfiguration` and `NluEngineConfigurationConvertible`
- Rust examples (replaced by interactive CLI).

## [0.57.2] - 2018-07-12
### Fixed
- Conflict with bindgen dependency

## [0.57.1] - 2018-07-09
### Changed
- Bump `snips-nlu-ontology` to `0.57.1`

### Fixed
- Crash when parsing implicit years before 1970

## [0.57.0] - 2018-06-08
### Changed
- Improve matching of synonyms
- Improve caching strategy for builtin entity parsing
- Improve intent classification
- Bump model version to `0.15.0`
- Bump `snips-nlu-ontology` to `0.57.0`

## [0.56.1] - 2018-05-18
### Changed
- Improve calibration of intent classification probabilities
- Update the `IntentParser` API and keep only `parse` method, while removing `get_intent` and `get_slots`
- DeterministicIntentParser: Replace tokenized out characters with whitespaces to improve matching

### Fixed
- DeterministicIntentParser: Fix issue with ranges of custom slots appearing after builtin slots

## [0.56.0] - 2018-05-03
### Changed
- Change ffi signatures
- Update swift project to Xcode 9.3
- Bump snips-nlu-ontology to `0.55.0`

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

[0.65.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.65.0...0.65.1
[0.65.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.64.4...0.65.0
[0.64.4]: https://github.com/snipsco/snips-nlu-rs/compare/0.64.3...0.64.4
[0.64.3]: https://github.com/snipsco/snips-nlu-rs/compare/0.64.2...0.64.3
[0.64.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.64.1...0.64.2
[0.64.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.64.0...0.64.1
[0.64.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.63.1...0.64.0
[0.63.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.63.0...0.63.1
[0.63.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.62.0...0.63.0
[0.61.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.61.1...0.61.2
[0.61.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.61.0...0.61.1
[0.62.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.61.0...0.62.0
[0.61.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.60.1...0.61.0
[0.60.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.60.0...0.60.1
[0.60.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.59.0...0.60.0
[0.59.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.58.3...0.59.0
[0.58.3]: https://github.com/snipsco/snips-nlu-rs/compare/0.58.2...0.58.3
[0.58.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.58.1...0.58.2
[0.58.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.58.0...0.58.1
[0.58.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.57.2...0.58.0
[0.57.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.57.1...0.57.2
[0.57.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.57.0...0.57.1
[0.57.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.56.1...0.57.0
[0.56.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.56.0...0.56.1
[0.56.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.55.2...0.56.0
[0.55.2]: https://github.com/snipsco/snips-nlu-rs/compare/0.55.1...0.55.2
[0.55.1]: https://github.com/snipsco/snips-nlu-rs/compare/0.55.0...0.55.1
[0.55.0]: https://github.com/snipsco/snips-nlu-rs/compare/0.54.0...0.55.0
