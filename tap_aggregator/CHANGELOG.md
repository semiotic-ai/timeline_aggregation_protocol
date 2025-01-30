# Changelog

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.2.0 to 0.3.0

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.3.0 to 0.4.0

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.5.1 to 0.6.0

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.8.0 to 1.0.0

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from * to 2.0.0

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 3.0.0 to 3.0.1
    * tap_graph bumped from 0.1.0 to 0.2.0

## [0.5.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.4.1...tap_aggregator-v0.5.0) (2025-01-30)


### ⚠ BREAKING CHANGES

* aggregate v2 receipts into v2 rav ([#274](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/274))

### Features

* aggregate v2 receipts into v2 rav ([#274](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/274)) ([df70b82](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/df70b82f9c1b39698817c164e177aca1a5c0df84))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 3.0.1 to 4.0.0
    * tap_graph bumped from 0.2.0 to 0.2.1

## [0.4.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.3.3...tap_aggregator-v0.4.0) (2025-01-28)


### ⚠ BREAKING CHANGES

* rename EIP712SignedMessage to Eip712SignedMessage
* create tap_eip712_message, tap_receipts and tap_graph ([#268](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/268))
* make manager generic over rav ([#267](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/267))
* manager generic over receipt ([#266](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/266))

### Code Refactoring

* create tap_eip712_message, tap_receipts and tap_graph ([#268](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/268)) ([3d35cac](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/3d35cac73159a89125051b5148a88efc63eb2193))
* make manager generic over rav ([#267](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/267)) ([1fc51a3](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1fc51a3ff9ce74027b47a0e6d026a5bedd9ca00c))
* manager generic over receipt ([#266](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/266)) ([25a3316](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/25a3316abd8596c3d5081d6b8f0507034c60d2a4))
* rename EIP712SignedMessage to Eip712SignedMessage ([0b0b59e](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0b0b59e380c9e2f04da2b28c26ccd0202f15a4a8))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 2.0.0 to 3.0.0

## [0.3.3](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.3.2...tap_aggregator-v0.3.3) (2024-12-27)


### Features

* accept grpc requests in tap-aggregator ([#253](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/253)) ([3c56018](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/3c56018a321736ff19103ea69015160c3647364b))


### Performance Improvements

* add rayon to verify signatures process ([#255](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/255)) ([cfa4a06](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/cfa4a0610e901f90ff7397fcaf1ba4ac633e774b))

## [0.3.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.2.0...tap_aggregator-v0.3.0) (2024-03-11)


### ⚠ BREAKING CHANGES

* update project structure
* update rav attributes to camel case
* replace aggregator mnemonic with private key ([#201](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/201))

### Features

* **aggregator:** Add support for multiple signers in input ([#211](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/211)) ([b16f23d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/b16f23d5481de65658b08544c083e2849821370e)), closes [#205](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/205)
* **core:** Supply EIP712 domain sep with prefilled version and name ([#210](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/210)) ([2ed564b](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/2ed564b9581c4eff2364fdc490f0f8d0022a6982))
* replace aggregator mnemonic with private key ([#201](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/201)) ([24583b4](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/24583b468a08527f7add79c71da0c5d56ab760c9))


### Bug Fixes

* listen on 0.0.0.0 instead of localhost ([#203](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/203)) ([5099ad1](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/5099ad166dda203cb9938c1fa417cf86d2215667))
* listen on 0.0.0.0 instead of localhost ([#204](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/204)) ([95d8ea6](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/95d8ea6bcd0e22e15dc8a2fee29c9b64abda978b))
* rebase main ([0c7bc1c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0c7bc1c9cebbb8dfad1c83067a1457380009925f))
* update rav attributes to camel case ([1b232e4](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1b232e4de230dc4922937fea2b489c7409ca2408))


### Code Refactoring

* update project structure ([70ee2c6](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/70ee2c67125653c25f479ee5f11e7c7e555078b7))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.7.0 to 0.8.0

## [0.2.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.1.6...tap_aggregator-v0.2.0) (2023-11-28)


### ⚠ BREAKING CHANGES

* rename "gateway" to "sender" everywhere

### Code Refactoring

* rename "gateway" to "sender" everywhere ([309f41f](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/309f41f879b51a1f2840ef0ed2552d7faa338b86)), closes [#188](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/188)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.6.0 to 0.7.0

## [0.1.5](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.1.4...tap_aggregator-v0.1.5) (2023-08-24)


### Bug Fixes

* **core:** Update alloy to v0.3.2 ([1ea0e7a](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1ea0e7a7617ec9c9b65c2a1ea0813cb2c2042882))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.5.0 to 0.5.1

## [0.1.4](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.1.3...tap_aggregator-v0.1.4) (2023-08-17)


### Features

* **tap-aggregator:** add default for key derive path arg ([3737f51](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/3737f51d5235981c868995e4d5b6798917341123))
* **tap-aggregator:** allow argument for key derive path to be used with mnemonic ([aec0a66](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/aec0a6628fff813e017c2e30d25530f81038dcd7))
* **tap-aggregator:** update to work with alloy changes ([9e94403](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/9e9440354ae73cd3491e72552ebc1877c3313509))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.4.0 to 0.5.0

## [0.1.1](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_aggregator-v0.1.0...tap_aggregator-v0.1.1) (2023-07-20)


### Features

* **aggregator:** add basic logging ([117b2f2](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/117b2f245f3d967cdc518c9435822a3f1fefdbd6)), closes [#145](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/145)
* **aggregator:** add prometheus metrics ([054f5bb](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/054f5bb415ad44501ebb3afe8ea31492ed88a130))


### Bug Fixes

* **aggregator:** revert default port change ([2f76f95](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/2f76f959e7576f32c4052dca94bf6e21a2f7f9eb))
* **aggregator:** short args removed ([47c7183](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/47c7183d922198b84e09e669cf5a86ed4f3581e2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_core bumped from 0.1.0 to 0.2.0

## 0.1.0 (2023-06-29)


### ⚠ BREAKING CHANGES

* **aggregator:** Warn list in JSON RPC response

### Features

* add aggregator service ([26e51a1](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/26e51a1d68fe51ae8c12c802f968d5bf2bcf5ca3))
* Added lib ([57c9ca2](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/57c9ca29d7c111e41fd1f5c7c776684aeeb03c26))
* Added lib ([feaa54b](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/feaa54b082f308abbf9f34060b14e2d535293885))
* added lib for tap_aggregator modules ([ba5840d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/ba5840d85bc567c4dcc01201bf8107b9936cbcd3))
* **aggregator:** basic API version mgmt ([68e4f35](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/68e4f352a98fcc8bd9da8b31944a6d8f73433b54))
* **aggregator:** HTTP limit settings ([8e81485](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/8e814854a9f45096c30e130d39304ad7ded49c65))
* **aggregator:** Supported versions in deprecation warning ([d067c66](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/d067c66d0d51f4539333f98ae19632317d497f58))
* **aggregator:** Warn list in JSON RPC response ([1ea269b](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1ea269b49fe106363a222204994c5e23f065d19e))
* TAP Showcase: Added integration tests for `tap_manager` and `tap_aggregator`. ([975b3c7](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/975b3c746b91c0cfe7a0dfc8a3361401bc70db28))
* TAP Showcase: Added integration tests for `tap_manager` and `tap_aggregator`. ([bcb6d82](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/bcb6d820a0f257c61ad83cf739726c2646885ce9))


### Bug Fixes

* **aggregator:** previous_rav ownership ([b422504](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/b42250476f01dcc70941544bce51ab9c57e763f0))
* **aggregator:** RAV and receipt timestamp checks ([faa3a8b](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/faa3a8b62aea95947a39b5c8a6799199fd8f88e8))
* **aggregator:** signature errors ([d29d2df](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/d29d2df4fce07c9646d77297c689c91304a35d79))
* **aggregator:** timestamp was based in millis, but should be nanoseconds ([354557f](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/354557ff9633f3ba0af34dc7a004f52f7e49862c))
* **aggregator:** warning codes ([67e8ff2](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/67e8ff2b797e1efcf5d03d1452d055855b784d50))
