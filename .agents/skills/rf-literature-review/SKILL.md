---
name: rf-literature-review
description: Evaluate published communications and propagation methods against linkbudget's actual capabilities and produce sourced, bounded production proposals. Use for literature discovery or refreshing research priorities, not implementation or release execution.
---

# Communications literature to a link-budget production proposal

## Prerequisites and boundaries

- Work in the linkbudget checkout. Read `AGENTS.md`, inspect Git status, and
  preserve unrelated work.
- Start with `README.md`, `docs/README.md`, and
  `docs/agent-operating-loop.md` for the implemented model and verification rules.
  Use `docs/literature-opportunities.md` as the existing research index when it
  exists; the skill does not depend on that optional document.
- A literature request permits public source retrieval, not paid API calls,
  purchased papers, measurement uploads, dependency installation, delegation,
  external publication, or changes to runtime behavior without authorization.

## Working sequence

1. Identify the actual link decision: required EIRP, receiver G/T, modulation/FEC,
   BER or packet error target, fade margin, availability, orbit geometry, or
   sensitivity. Keep hardware-block lineups in gainlineup, S-parameter analysis in
   touchstone, and scalar conversions in rfconversions.
2. Read `src/lib.rs`, then the owning module and relevant public scenario tests.
   Consult the indexed physics guides before proposing a replacement formula.
   Record the Git baseline and distinguish existing capabilities from gaps.
3. Find primary papers or normative standards. Verify authors/issuing body,
   title, year, DOI or document identifier, and exact standard revision through
   publisher, institutional, standards-body, or bibliographic metadata. A nearby
   search result or a newer revision is not automatically the intended source.
4. Inspect the relevant equations, tables, and assumptions. Prefer legitimate
   open institutional/author copies. Use an available PDF-reading skill or
   existing tooling without implicitly installing dependencies. Record physical
   pages, sections, and unread equations; label metadata-only evidence honestly.
5. Translate each candidate into one bounded production slice. State:
   - the user-visible link-closure decision it improves and the owning module;
   - input data, reference planes, units, and validity regime;
   - numerical failure behavior and dependencies;
   - an independent published or analytic acceptance case plus a complete public
     workflow; an inverse/round trip alone is not a reference;
   - what remains an approximation and which adjacent features are deferred.
6. Audit the communications contract explicitly: information versus coded bit
   rate, symbol rate, FEC rate, Es/N0 versus Eb/N0, noise bandwidth versus channel
   bandwidth, dBm versus dBW, and absolute versus excess noise temperature.
   Name the error criterion, frame length, channel assumptions, decoder assumptions,
   and implementation margin for any sourced waveform-performance profile.
7. For propagation proposals, separate deterministic attenuation from statistical
   availability. Record location, elevation, polarization, climatic data,
   probability/time basis, recommendation revision, and coefficient provenance.
   Do not treat a hand-entered rain loss as an annual availability model.
8. Rank by decision value, evidence/data availability, integration cost, and
   validation cost. Prefer a small sourced profile or one validated propagation
   path over a modem simulator, generic optimizer, or global weather engine.
9. Extend `docs/literature-opportunities.md` when present. Otherwise extend the
   nearest maintained physics guide or create a research page for a substantive
   review and link it from `docs/README.md`. Clearly label proposed behavior;
   research must not silently rewrite the implemented model's claims.

## Verification and common failures

- Check source identity, local documentation links, the complete intended diff,
  and `git diff --check`. Use `uv run` for Python calculations; do not add a new
  test suite merely for a research document. Keep any required skill-owned helper
  inside this folder and resolve it relative to the skill directory.
- Treat existing `tests/readme_examples.rs`, `tests/scenarios.rs`, and
  `tests/receiver_physics.rs` as baseline evidence, not proof of unimplemented
  proposals. Separately authorized implementations follow repository gates.
- A coding-gain shift is not a measured FEC curve; a Shannon limit is not an
  implementable modem threshold; BER, FER, PER, and outage are different targets.
- Monte Carlo precision does not remove model uncertainty. Preserve correlations
  and distinguish sampling error, input uncertainty, and model discrepancy.
- A finite tiny BER result is not proof of tail accuracy. Check Gaussian-tail
  formulas and inverses against independent references and physical limits.
- If access or extraction fails, narrow the evidence claim or find a legitimate
  alternate source. Do not bypass access controls, invent citations, or copy
  downloadable reference code without a license review.

## Completion and recovery

Report the best next bounded feature, sources and access depth, verification,
assumptions, and remaining uncertainty. Preserve the existing model until a
separately authorized implementation is validated. Research alone does not
permit commits, pushes, releases, or resetting unrelated work. Remove or demote
recommendations whose evidence fails or whose capability already exists.
