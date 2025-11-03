# dCBOR Diagnostic Parser and Composer

<!--Guidelines: https://github.com/BlockchainCommons/secure-template/wiki -->

### _by Wolf McNally_

---

This crate provides tools for parsing and composing the [CBOR diagnostic notation](https://datatracker.ietf.org/doc/html/rfc8949#name-diagnostic-notation) into [dCBOR (deterministic CBOR)](https://datatracker.ietf.org/doc/draft-mcnally-deterministic-cbor/) data items.

It is intended for use in testing, debugging, the `dcbor` command line tool, and other scenarios where a human-readable representation of dCBOR is useful. It is not optimized for performance and should not be used in production environments where binary dCBOR is expected.

The primary functions provided are:

- `parse_dcbor_item`: Parses a string in CBOR diagnostic notation into a `CBOR` object.
- `parse_dcbor_item_partial`: Parses the first item in a string and reports how
  many bytes were consumed.
- `compose_dcbor_array`: Composes a `CBOR` array from a slice of strings representing dCBOR items in diagnostic notation.
- `compose_dcbor_map`: Composes a `CBOR` map from a slice of strings representing the key-value pairs in dCBOR diagnostic notation.

The syntactical types supported are:

| Type                | Example(s)                                                  |
| ------------------- | ----------------------------------------------------------- |
| Boolean             | `true`<br>`false`                                           |
| Null                | `null`                                                      |
| Integers            | `0`<br>`1`<br>`-1`<br>`42`                                  |
| Floats              | `3.14`<br>`-2.5`<br>`Infinity`<br>`-Infinity`<br>`NaN`      |
| Strings             | `"hello"`<br>`"🌎"`                                          |
| ISO-8601 Dates      | `2023-10-01T12:00:00Z`<br>`2023-10-01`                      |
| Hex Byte Strings    | `h'68656c6c6f'`                                             |
| Base64 Byte Strings | `b64'AQIDBAUGBwgJCg=='`                                     |
| Tagged Values       | `1234("hello")`<br>`5678(3.14)`                             |
| Name-Tagged Values  | `tag-name("hello")`<br>`tag-name(3.14)`                     |
| Known Values        | `'1'`<br>`'isA'`                                            |
| Unit Known Value    | `Unit`<br>`''`<br>`'0'`                                     |
| URs                 | `ur:date/cyisdadmlasgtapttl`                                |
| Arrays              | `[1, 2, 3]`<br>`["hello", "world"]`<br>`[1, [2, 3]]`        |
| Maps                | `{1: 2, 3: 4}`<br>`{"key": "value"}`<br>`{1: [2, 3], 4: 5}` |

## Parsing Named Tags and Uniform Resources (URs)

A [Uniform Resource (UR)](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) is a URI representation of tagged dCBOR, where the tag is represented as a text type component. The last component of the UR is the untagged CBOR encoded as ByteWords, including a CRC-32 checksum in the last eight letters.

To parse named tags and URs, the correspondence between the tag name (UR type) and the integer CBOR tag value must be known. This is done by using the `with_tags!` macro to access the global tags registry. Clients wishing to parse named tags and URs must register the CBOR tag value and its corresponding name in the global tags registry. The [`dcbor`](https://crates.io/crates/dcbor) crate only registers one tag and name for `date` (tag 1). The [`bc-tags`](https://crates.io/crates/bc-tags) crate registers many more. See the `register_tags` functions in these crates for examples of how to register your own tags.

## Getting Started

```toml
[dependencies]
dcbor-parse = "0.6.0"
```

## Version History

### 0.6.0 - November 3, 2025

- Align to dependencies.

### 0.5.0 - October 20, 2025

- Format.
- Align to dependencies.

### 0.4.0 - September 16, 2025

- Align to dependencies.
- Remove anyhow feature from dcbor dependency.
- Clean up test code formatting and API usage.

### 0.3.0 - July 3, 2025

- Add ISO-8601 date literal parsing support
- Add duplicate map key detection with proper error handling
- Add simplified-patterns feature flag for IDE compatibility
- Align to dependencies
- Add comprehensive examples and tests for new features

## Related Projects

- [dCBOR Library](https://github.com/BlockchainCommons/bc-dcbor-rust)
- [dCBOR-CLI Reference App](https://github.com/BlockchainCommons/dcbor-cli)

## Status - Community Review

`dcbor` is now considered production-ready but the parser/composer is new and may require further refinement. We are looking for feedback from the community to help us improve the API and functionality. We are also looking for additional test cases to ensure that the parser/composer works correctly in all scenarios.

Let us know if the API meets your needs, and the functionality is easy to use. Comments can be posted [to the Gordian Developer Community](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions/116).

See [Blockchain Commons' Development Phases](https://github.com/BlockchainCommons/Community/blob/master/release-path.md).

## Financial Support

`dcbor` is a project of [Blockchain Commons](https://www.blockchaincommons.com/). We are proudly a "not-for-profit" social benefit corporation committed to open source & open development. Our work is funded entirely by donations and collaborative partnerships with people like you. Every contribution will be spent on building open tools, technologies, and techniques that sustain and advance blockchain and internet security infrastructure and promote an open web.

To financially support further development of `dcbor` and other projects, please consider becoming a Patron of Blockchain Commons through ongoing monthly patronage as a [GitHub Sponsor](https://github.com/sponsors/BlockchainCommons). You can also support Blockchain Commons with bitcoins at our [BTCPay Server](https://btcpay.blockchaincommons.com/).

## Contributing

We encourage public contributions through issues and pull requests! Please review [CONTRIBUTING.md](./CONTRIBUTING.md) for details on our development process. All contributions to this repository require a GPG signed [Contributor License Agreement](./CLA.md).

### Discussions

The best place to talk about Blockchain Commons and its projects is in our GitHub Discussions areas.

[**Gordian Developer Community**](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions). For standards and open-source developers who want to talk about interoperable wallet specifications, please use the Discussions area of the [Gordian Developer Community repo](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions). This is where you talk about Gordian specifications such as [Gordian Envelope](https://github.com/BlockchainCommons/Gordian/tree/master/Envelope#articles), [bc-shamir](https://github.com/BlockchainCommons/bc-shamir), [Sharded Secret Key Reconstruction](https://github.com/BlockchainCommons/bc-sskr), and [bc-ur](https://github.com/BlockchainCommons/bc-ur) as well as the larger [Gordian Architecture](https://github.com/BlockchainCommons/Gordian/blob/master/Docs/Overview-Architecture.md), its [Principles](https://github.com/BlockchainCommons/Gordian#gordian-principles) of independence, privacy, resilience, and openness, and its macro-architectural ideas such as functional partition (including airgapping, the original name of this community).

[**Gordian User Community**](https://github.com/BlockchainCommons/Gordian/discussions). For users of the Gordian reference apps, including [Gordian Coordinator](https://github.com/BlockchainCommons/iOS-GordianCoordinator), [Gordian Seed Tool](https://github.com/BlockchainCommons/GordianSeedTool-iOS), [Gordian Server](https://github.com/BlockchainCommons/GordianServer-macOS), [Gordian Wallet](https://github.com/BlockchainCommons/GordianWallet-iOS), and [SpotBit](https://github.com/BlockchainCommons/spotbit) as well as our whole series of [CLI apps](https://github.com/BlockchainCommons/Gordian/blob/master/Docs/Overview-Apps.md#cli-apps). This is a place to talk about bug reports and feature requests as well as to explore how our reference apps embody the [Gordian Principles](https://github.com/BlockchainCommons/Gordian#gordian-principles).

[**Blockchain Commons Discussions**](https://github.com/BlockchainCommons/Community/discussions). For developers, interns, and patrons of Blockchain Commons, please use the discussions area of the [Community repo](https://github.com/BlockchainCommons/Community) to talk about general Blockchain Commons issues, the intern program, or topics other than those covered by the [Gordian Developer Community](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions) or the
[Gordian User Community](https://github.com/BlockchainCommons/Gordian/discussions).

### Other Questions & Problems

As an open-source, open-development community, Blockchain Commons does not have the resources to provide direct support of our projects. Please consider the discussions area as a locale where you might get answers to questions. Alternatively, please use this repository's [issues](./issues) feature. Unfortunately, we can not make any promises on response time.

If your company requires support to use our projects, please feel free to contact us directly about options. We may be able to offer you a contract for support from one of our contributors, or we might be able to point you to another entity who can offer the contractual support that you need.

### Credits

The following people directly contributed to this repository. You can add your name here by getting involved. The first step is learning how to contribute from our [CONTRIBUTING.md](./CONTRIBUTING.md) documentation.

| Name              | Role                     | Github                                           | Email                                 | GPG Fingerprint                                    |
| ----------------- | ------------------------ | ------------------------------------------------ | ------------------------------------- | -------------------------------------------------- |
| Christopher Allen | Principal Architect      | [@ChristopherA](https://github.com/ChristopherA) | \<ChristopherA@LifeWithAlacrity.com\> | FDFE 14A5 4ECB 30FC 5D22 74EF F8D3 6C91 3574 05ED  |
| Wolf McNally      | Lead Researcher/Engineer | [@WolfMcNally](https://github.com/wolfmcnally)   | \<Wolf@WolfMcNally.com\>              | 9436 52EE 3844 1760 C3DC  3536 4B6C 2FCF 8947 80AE |

## Responsible Disclosure

We want to keep all of our software safe for everyone. If you have discovered a security vulnerability, we appreciate your help in disclosing it to us in a responsible manner. We are unfortunately not able to offer bug bounties at this time.

We do ask that you offer us good faith and use best efforts not to leak information or harm any user, their data, or our developer community. Please give us a reasonable amount of time to fix the issue before you publish it. Do not defraud our users or us in the process of discovery. We promise not to bring legal action against researchers who point out a problem provided they do their best to follow the these guidelines.

### Reporting a Vulnerability

Please report suspected security vulnerabilities in private via email to ChristopherA@BlockchainCommons.com (do not use this email for support). Please do NOT create publicly viewable issues for suspected security vulnerabilities.

The following keys may be used to communicate sensitive information to developers:

| Name              | Fingerprint                                       |
| ----------------- | ------------------------------------------------- |
| Christopher Allen | FDFE 14A5 4ECB 30FC 5D22 74EF F8D3 6C91 3574 05ED |

You can import a key by running the following command with that individual’s fingerprint: `gpg --recv-keys "<fingerprint>"` Ensure that you put quotes around fingerprints that contain spaces.
