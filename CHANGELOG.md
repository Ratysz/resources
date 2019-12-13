# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--## [Unreleased](https://github.com/Ratysz/resources/compare/0.2.1..HEAD)-->

## [0.2.1](https://github.com/Ratysz/resources/compare/0.2.0..0.2.1) - 2019-12-13
### Changed
- Removed `RwLock` reinvention in favor of implementations provided by `parking_lot`.
### Fixed
- Example in README.md.

## [0.2.0](https://github.com/Ratysz/resources/compare/0.1.0..0.2.0) - 2019-12-13
### Added
- Full documentation.
- LICENSE.md, to link to with badges.
### Changed
- `Resources::remove()` now returns an option rather than a result.
- README.md now mirrors crate level docs.

## [0.1.0](https://github.com/Ratysz/resources/releases/tag/0.1.0)  - 2019-12-13
### Added
- Initial release.