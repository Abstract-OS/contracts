# Publish guide

Publishing new abstract versions.

## Utils

Two utils packages (`abstract-ica` and `abstract-macros`) are used by a lot of packages and are considered stable. They are published manually.

## Packages

Ideally only update the packages that have changed. If you are unsure, update all packages.  
By changing the version in the Cargo workspace you change the version of all contracts and `abstract-boot`.
`abstract-boot` is a wrapper package around all the abstract contracts and is used extensively in testing.

New releases of `abstract_core`, `abstract-sdk` or `abstract-testing` should be reflected in the Cargo workspace
file.

1. `abstract_core`
2. `abstract-testing`
3. `abstract-sdk`
4. All contracts in `./contracts`
5. `abstract-boot`
6. `abstract-api`, `abstract-app` and `abstract-ibc-host`
