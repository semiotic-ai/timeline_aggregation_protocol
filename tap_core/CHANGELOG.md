# Changelog

* The following workspace dependencies were updated
  * dependencies
    * tap_graph bumped from 0.1.0 to 0.2.0

## [4.0.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v3.0.1...tap_core-v4.0.0) (2025-01-30)


### ⚠ BREAKING CHANGES

* relax manager constraints for ravs ([#275](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/275))

### Code Refactoring

* relax manager constraints for ravs ([#275](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/275)) ([9fd4beb](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/9fd4beb4a8c55114a4e33b851b86d80b71d42a00))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * tap_receipt bumped from 0.1.0 to 1.0.0
    * tap_graph bumped from 0.2.0 to 0.2.1

## [3.0.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v2.0.0...tap_core-v3.0.0) (2025-01-28)


### ⚠ BREAKING CHANGES

* rename EIP712SignedMessage to Eip712SignedMessage
* create tap_eip712_message, tap_receipts and tap_graph ([#268](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/268))
* make manager generic over rav ([#267](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/267))
* manager generic over receipt ([#266](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/266))
* remove unused escrow handler methods ([#264](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/264))

### Code Refactoring

* create tap_eip712_message, tap_receipts and tap_graph ([#268](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/268)) ([3d35cac](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/3d35cac73159a89125051b5148a88efc63eb2193))
* make manager generic over rav ([#267](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/267)) ([1fc51a3](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1fc51a3ff9ce74027b47a0e6d026a5bedd9ca00c))
* manager generic over receipt ([#266](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/266)) ([25a3316](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/25a3316abd8596c3d5081d6b8f0507034c60d2a4))
* remove unused escrow handler methods ([#264](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/264)) ([e5511ff](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/e5511ff7a5cc4753782f718f4517a0a0a8d9db56))
* rename EIP712SignedMessage to Eip712SignedMessage ([0b0b59e](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0b0b59e380c9e2f04da2b28c26ccd0202f15a4a8))

## [2.0.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v1.0.0...tap_core-v2.0.0) (2024-10-30)


### ⚠ BREAKING CHANGES

* add context to checks
* add retryable errors to checks

### Features

* add context to checks ([58a6a52](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/58a6a52eba8152ee3add27d10e59b9e10ab9a5f4))
* add retryable errors to checks ([51f04cb](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/51f04cb0dbe7387ffd94f16eb59bbeb1c6c51680))
* expected_rav to become result&lt;rav,error&gt; ([70fa8fa](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/70fa8fa116c32b074f5c691979de02177b385560))
* make error public access on failed receipt ([7180b86](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/7180b86e2c5ef5e2009518d6cd2ed8fe16b337eb))
* obtain invalid receipts ([101dc5e](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/101dc5eea45baf143c5135fe66bbb5f62dfa9109))


### Bug Fixes

* Remove obtain_invalid_receipts ([ad255dc](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/ad255dcbb4a5ea101c575038534312b59e12d693))

## [1.0.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.8.0...tap_core-v1.0.0) (2024-03-27)


### ⚠ BREAKING CHANGES

* create docs and readme.

### Documentation

* create docs and readme. ([4ebf258](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/4ebf2584441514166fd9e91999b1710af1d54524))

## [0.8.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.7.0...tap_core-v0.8.0) (2024-03-11)


### ⚠ BREAKING CHANGES

* update project structure
* implement unique and timestamp check hard coded
* remove stateful checks
* remove timestamp check from manager
* update rav attributes to camel case
* use single executor for manager and auditor
* convert receipt and rav storage into executor
* use typestate for receivedreceipt
* split read and write storage adapters

### Features

* add serde to ReceivedReceipt ([b13bedf](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/b13bedf4f145fe0209c48d4d51630233ad71b4b8))
* **aggregator:** Add support for multiple signers in input ([#211](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/211)) ([b16f23d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/b16f23d5481de65658b08544c083e2849821370e)), closes [#205](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/205)
* **core:** Supply EIP712 domain sep with prefilled version and name ([#210](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/210)) ([2ed564b](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/2ed564b9581c4eff2364fdc490f0f8d0022a6982))
* implement unique and timestamp check hard coded ([7c3f5a9](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/7c3f5a97679b7fcb3cd1150517e6bd77585255e0))


### Bug Fixes

* add debug to messageid ([fbdd328](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/fbdd328f4170d58800b62deed833493b728afb68))
* checks need to be send and sync ([78ae2a8](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/78ae2a89e395a49a186eef62a26c5f51ca279735))
* compile tests ([06cf24c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/06cf24cac1b4e50bcd8234557e09556b7de75007))
* implement receipt delete to mock ([50058f7](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/50058f7043303bbb914de7808b641b39800e0ede))
* missing merge conflicts ([8c95954](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/8c9595456bad9c334b539d0363de884239f14157))
* rebase main ([0c7bc1c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0c7bc1c9cebbb8dfad1c83067a1457380009925f))
* remove EscrowAdapter dependency ([3045f61](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/3045f61c2ce556ab13bf9268102c9af0387b6226))
* update rav attributes to camel case ([1b232e4](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1b232e4de230dc4922937fea2b489c7409ca2408))
* update timestamp check only after rav update ([bfb1eb0](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/bfb1eb07aa2d26f611ce14d4d0b57fddb9da2ea6))
* use the correct timestamp check ([1b1d19c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1b1d19ca00244f01fb45c8d62398dfe16ddac7bb))


### Reverts

* rename back to adapters ([42bd2df](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/42bd2dfce16d12753ade4577cb0ee47b3d82dbce))


### Code Refactoring

* convert receipt and rav storage into executor ([467c917](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/467c917af61733e3c2cbf3823c4377091179980b))
* remove stateful checks ([1044f53](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1044f534be5e35a387d32e6b8bd86e86af188b66))
* remove timestamp check from manager ([0a3b983](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0a3b983b5f20b27c0afdc02d9db682481e7493a9))
* split read and write storage adapters ([2e681a5](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/2e681a529d0c03e2fe455f64692e23024ad98073))
* update project structure ([70ee2c6](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/70ee2c67125653c25f479ee5f11e7c7e555078b7))
* use single executor for manager and auditor ([6794fbb](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/6794fbb6a7d02774008d57d62ccc10564cf952c1))
* use typestate for receivedreceipt ([89b5d94](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/89b5d941a37c475cb47c768f2618b902cf8908c5))

## [0.7.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.6.0...tap_core-v0.7.0) (2023-11-28)


### ⚠ BREAKING CHANGES

* add limit to receipts retrieve
* create_rav_request() returns invalid ReceivedReceipt
* rename "gateway" to "sender" everywhere

### Features

* add limit to receipts retrieve ([0ce2aab](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0ce2aabca5cf20bb531578e3a92d33ed0a2d4c17))
* create_rav_request() returns invalid ReceivedReceipt ([5bb9001](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/5bb90015dc636aa4c0916e7072724ae7fc03e8e4))


### Bug Fixes

* mutable manager.remove_obsolete_receipts ([ca1a01e](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/ca1a01e8571f9c65a6b58efbb4964b568c03e6ac))


### Code Refactoring

* rename "gateway" to "sender" everywhere ([309f41f](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/309f41f879b51a1f2840ef0ed2552d7faa338b86)), closes [#188](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/188)

## [0.6.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.5.1...tap_core-v0.6.0) (2023-10-12)


### ⚠ BREAKING CHANGES

* **core:** 

### Code Refactoring

* **core:** manager arg slice instead of vec ([ee95e94](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/ee95e9492bdfeeabadc2b0510940abc082b34cd4))

## [0.5.1](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.5.0...tap_core-v0.5.1) (2023-08-24)


### Bug Fixes

* Update alloy to v0.3.2 ([715a04c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/715a04c1d9b9622e0bb846830f8dd782062901c7))

## [0.5.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.4.0...tap_core-v0.5.0) (2023-08-17)


### ⚠ BREAKING CHANGES

* **tap-core:** receipts, RAVs, and eip712-signed-message now use alloy primitive address

### Features

* **tap-core-bench:** update to work with alloy changes ([43c5e6c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/43c5e6c24218b9c5138cabec50262715f6b34124))
* **tap-core:** changes eip712 and address components to alloy library ([a99af2f](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/a99af2f8d64ad2d1a4d175de5e7789254fb2c36f))

## [0.4.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.3.0...tap_core-v0.4.0) (2023-08-08)


### ⚠ BREAKING CHANGES

* **core:** All ReceiptChecksAdapter trait method now return Result. Makes it possible to return Adapter errors besides just check failures.
* All instances of the word "collateral" are replaced with "escrow".

### Code Refactoring

* **core:** make ChecksAdapter return Result ([0983c27](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/0983c27c40d2061a51687fa0517add2593d3b934))
* Replace "collateral" with "escrow" ([6f9d0c7](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/6f9d0c7a54fa445f1e058a65bcfccdef06cf9d02))

## [0.3.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.2.0...tap_core-v0.3.0) (2023-07-31)


### ⚠ BREAKING CHANGES

* **core:** all the traits are now async

### Features

* serde for ReceivedReceipt ([1765b5d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/1765b5d101e73ff0085e6246252b47e4d5e98890))


### Code Refactoring

* **core:** make it all async ([30ca4ba](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/30ca4ba7da05959d380b653de0307af273c4733d))

## [0.2.0](https://github.com/semiotic-ai/timeline-aggregation-protocol/compare/tap_core-v0.1.0...tap_core-v0.2.0) (2023-07-20)


### ⚠ BREAKING CHANGES

* **tap_core:** Preserve adapter error type using anyhow ([#133](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/133))
* **receipt_storage_adapter:** all functions unnecessary to the TAP manager have been removed from the `ReceiptStorageAdapter` trait.
* **rav_storage_adapter:** the RavStorageAdapter public trait has changed

### Features

* **manager:** add receipts_auto_delete ([#130](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/130)) ([37bc8e1](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/37bc8e1212fb525e15be465bdc0dba4e5349c025))
* **receipt_storage_adapter:** Use RangeBounds ([663a8ba](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/663a8ba7f0afb74cc17e4eb58802282771cb0bca))
* **tap_core:** Preserve adapter error type using anyhow ([#133](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/133)) ([77abbd8](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/77abbd82d5e0c75cd04b1bce55360a312f036bac))


### Bug Fixes

* **tap_manager:** receipt auditor min_timestamp set incorrectly ([743ac7c](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/743ac7cd252a3723ad01fbd4a1b54cbaf68267af))


### Code Refactoring

* **rav_storage_adapter:** simplify trait ([6af9471](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/6af9471bea8aeafd415b4f3e7b92a1eb6bec135b))
* **receipt_storage_adapter:** prune trait ([caef197](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/caef197261f16187402f69091ffea843e6aef970))

## 0.1.0 (2023-06-29)


### ⚠ BREAKING CHANGES

* **tap-manager:** RAV Request definition is updated to include optional previous RAV
* **receipts:** Updates the receipt checks adapter api
* **signed-message:** Updates signed message API
* **allocation-adapter:** removes allocation adapter
* **adapters:** existing adapter trait definitions are updated

### Features

* **adapter-mocks:** alters adapters to use references (arc) to allow sharing common resources ([03e7668](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/03e7668e3c59d27e6cfc869a3a35ad1434d18d6d))
* **adapters:** adds functionality to retrieve and delete receipts within a range ([4143ac6](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/4143ac6293751a0e837709bba43d4fc600911bcc))
* **adapters:** split adapters into storage and check adapters ([39c1c82](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/39c1c827447aceb938ad03643bb7bf08ff330cae))
* **core:** add `verify` to EIP712 signed msg ([a7e3e7d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/a7e3e7d18044dbe6937cf725376167171fb177b1))
* **receipt-storage:** Adds functionality to update receipt in storage by id ([eb4f8ba](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/eb4f8bae233406b6c5d25def4de1d628d7860b1e))
* **receipts:** Updates checking mechanisms and adds auditor to complete checks ([8d0088d](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/8d0088d6fbc83416737cf33c3e305412741c8ec8))
* **signed-message:** Updates library to use ether-rs for wallet, address, key, signature, and verification ([7f1cb85](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/7f1cb8586e7577221008588cacd5cc6ad47d1c83))
* **tap-manager:** Adds a tap manager for handling receipt and rav validation and storage ([3786042](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/378604263a91d3abd9fdad1dd978f1ba715f7aca))


### Bug Fixes

* **allocation-adapter:** remove obsolete trait ([957f3f9](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/957f3f99efa8b5ebe7f61024f96905b0448c4bed))
* **receipt-errors:** updates receipt errors to work with adapter trait api ([fc121bf](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/fc121bf21bee2b8bdf0d1db026e97afb2844d75b))
* **receipts:** adds receipt ID to check unique to allow checking after receipt is in storage ([5072fb9](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/5072fb9ba614d58bcc712778deb451eaefbc993f))
* **tap-manager:** adds an error when receipt timestamp window is inverted ([e21d7d9](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/e21d7d941b8b298bdda5dc8143b3163f65ca1e85))
* **tap-manager:** receipts being used after being added to RAV ([efd88a2](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/efd88a214a3737b7bb201cabaf4037284ec5d547))
* verification benchmarks ([#114](https://github.com/semiotic-ai/timeline-aggregation-protocol/issues/114)) ([96cdf24](https://github.com/semiotic-ai/timeline-aggregation-protocol/commit/96cdf24db98a715cec654ff77de1837ba36f81a4))
