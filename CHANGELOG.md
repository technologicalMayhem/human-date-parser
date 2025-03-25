# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The library no longer uses the local time zone and instead uses naive times.
  Handling of time zones is left up to the consumer of the library.
- Internal: Input text is not being parsed into a custom AST before being
  processed. This should make it easier to reason about how the code works.