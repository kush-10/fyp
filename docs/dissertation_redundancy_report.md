# Dissertation Redundancy Analysis
*Benchmarking Symmetric Cryptography in Zero-Knowledge Virtual Machines — Kushal Patel, URN 6779705*

---

## Executive Summary

This report identifies 13 recurring themes across the dissertation that appear more than once, often restating identical figures, arguments, or caveats without adding new context. Each repetition is categorised by severity with a clear recommendation: where to keep content and where to cut or replace with a cross-reference.

> **Overall word-saving estimate:** Addressing the HIGH and MEDIUM redundancies alone could reduce body text by approximately 10–15% without losing any substantive content or evidence.

| Severity | Count | Primary Effect |
|----------|-------|----------------|
| HIGH | 3 | Identical numbers / caveats repeated verbatim across 4–8 chapters |
| MEDIUM | 6 | Same argument restated in 3–4 chapters; one clear home exists |
| LOW | 4 | Mild duplication; a cross-reference or one-line cut resolves it |

---

## How to Read This Report

Each entry follows this structure:
- **Theme** — what is repeated
- **Severity** — HIGH / MEDIUM / LOW
- **Occurrences** — every location with a representative quote
- **Keep** — where the content should stay and why
- **Cut / Reduce** — where to delete or replace with a cross-reference

> **Note:** "Cut" does not mean the idea is wrong — it means it has already been made in a better location, and the repetition weakens the argument by signalling uncertainty that the reader absorbed it the first time.

---

## HIGH Severity — Cut Recommended

---

### R1 — LowMC vs AES result (34.85× slower)

| Chapter / Section | Representative Quote |
|---|---|
| Abstract | "optimised LowMC remains about 34.85× slower than AES on proving time" |
| Ch.1 §1.6 Key Results | "lowmc-r0-optimised is about 34.85× slower than aes-r0 on proving time" |
| Ch.5 §5.2 | "lowmc-r0-optimised is about 34.85× slower than aes-r0 on proving time (517.209 s vs 14.843 s)" |
| Ch.11 §11.7 | "lowmc-r0-optimised proving time is ~34.85× aes-r0 in the report benchmark run" |
| Ch.15 Conclusion | "lowmc-r0-optimised is about 34.85× slower than aes-r0 on proving time (517.209 s vs 14.843 s)" |

✅ **Keep:** Ch.5 §5.2 (primary analysis) and Ch.15 (conclusion summary). Keep a brief high-level mention in the Abstract.

✂️ **Cut:** Ch.1 §1.6 is a near-verbatim preview that duplicates Ch.5 before the reader has any context. Ch.11 §11.7 adds no new analysis — replace with a cross-reference to Ch.5.

---

### R2 — LowMC optimisation reduced proving time by ~25.59%

| Chapter / Section | Representative Quote |
|---|---|
| Abstract | "LowMC optimisation reduces proving time by about 25.59% relative to its baseline snapshot" |
| Ch.1 §1.6 | "LowMC optimisation was still useful locally, reducing proving time by about 25.59%" |
| Ch.5 §5.2 (Table 5.1) | "Prove seconds delta: -25.59%" |
| Ch.11 §11.7 | "optimisation reduced LowMC proving time by ~25.59% in the baseline-vs-optimised snapshot" |
| Ch.15 Conclusion | "LowMC optimisation improved local performance materially (~25.59% proving-time reduction)" |

✅ **Keep:** Table 5.1 in Ch.5 (primary data) and Ch.15 (conclusion). A high-level mention in the Abstract is acceptable.

✂️ **Cut:** Ch.1 §1.6 preview and Ch.11 §11.7 both restate the same number without new analysis. Replace both with a cross-reference to Table 5.1.

---

### R3 — Single-trial / N=1 caveat

| Chapter / Section | Representative Quote |
|---|---|
| Ch.1 §1.7 Limitations Preview | "most final values come from one trial per target" |
| Ch.3 §3.4 | "The report benchmark campaign uses N=1 per target" |
| Ch.3 §3.5 | "Threats to validity — single trial, noise" |
| Ch.11 §11.1 | "one trial per target in this evaluation" |
| Ch.11 §11.5 | "noisy in single-trial campaign" |
| Ch.11 §11.9 | "The final campaign is suitable for evidence traceability…not for high-confidence inference" |
| Ch.12 §12.1 | "headline results are from single-trial runs" |
| Ch.15 §15.1 (Table 15.1) | "single-trial caveat" |

✅ **Keep:** Ch.3 §3.4 (full rationale), Ch.12 §12.1 (limitations), and a single introductory note in Ch.11 §11.1.

✂️ **Cut:** The caveat is restated in almost every section of Ch.11. One paragraph in §11.1 is enough; subsequent sections should say "as noted in §3.4" rather than re-explaining the point from scratch.

---

## MEDIUM Severity — Reduce or Cross-Reference

---

### R4 — Encrypt-then-MAC / verify-before-decrypt order

| Chapter / Section | Representative Quote |
|---|---|
| Ch.6 §6.1 | "appended to the AES-CTR ciphertext in an Encrypt-then-MAC pattern" |
| Ch.7 §7.4 | "Why Encrypt-then-MAC Runs in the VM" |
| Ch.8 §8.3 | "verify-before-decrypt order prevents processing unauthenticated ciphertext" |
| Ch.9 §9.2 | "encrypt and tag generation stay inside the guest" |
| Ch.9 §9.3 | "verify-before-decrypt contract" |

✅ **Keep:** Ch.7 §7.4 (primary rationale) and Ch.8 §8.3 (design specification).

✂️ **Cut:** Ch.6 §6.1 introduces the pattern before it is motivated — move this detail to Ch.7. Ch.9 §9.2–9.3 repeat the verify-before-decrypt logic already fully stated in §8.3; one sentence plus a reference is enough.

---

### R5 — Three-layer separation (data auth / computation correctness / transport identity)

| Chapter / Section | Representative Quote |
|---|---|
| Ch.7 §7.2 (Threat-Driven Requirements) | "AES-CTR addresses the first threat… HMAC… Receipt verification…" |
| Ch.7 §7.3 (Layer Separation) | "Data authentication / Computation correctness / Transport/sender identity" |
| Ch.7 §7.6 (Implementation Scope) | "current claims cover confidentiality and integrity… not sender identity" |
| Ch.12 §12.4 | "does not add external proof-transport signature layer" |

✅ **Keep:** Ch.7 §7.3 is the right home for the full explanation. Ch.12 §12.4 is appropriate as a limitations statement.

✂️ **Cut:** §7.2 partially duplicates §7.3. The three layers are explained in both; §7.2 should be reduced to stating the threats only, deferring the layer-resolution discussion to §7.3.

---

### R6 — RISC Zero host/guest split explanation

| Chapter / Section | Representative Quote |
|---|---|
| Ch.2 §2.2 | "Guest: deterministic RISC-V program… Host: prepares inputs, invokes proving" |
| Ch.3 §3.1 | "Guest programs execute cryptographic logic… host programs serialize inputs, trigger proving" |
| Ch.8 §8.4 | "host serializes mode-specific inputs… guest performs deterministic cryptographic computation" |
| Ch.9 §9.2 / §9.3 | "Guest Responsibilities / Host Responsibilities" |

✅ **Keep:** Ch.2 §2.2 (background definition) and Ch.9 §9.2–9.3 (implementation-specific detail).

✂️ **Cut:** Ch.3 §3.1 repeats the split without adding any experimental context. Ch.8 §8.4 largely restates what §9.2–9.3 covers in more detail — shorten to a pointer.

---

### R7 — AES-CTR mode description (block cipher → keystream → XOR)

| Chapter / Section | Representative Quote |
|---|---|
| Ch.4 §4.2 | "AES does this by maintaining a compact 16-byte state and applying repeated rounds…" |
| Ch.6 §6.1 | "CTR mode converts the AES block cipher into a stream cipher: the IV and a counter are concatenated…" |

✅ **Keep:** Ch.4 §4.2 for the cipher description; Ch.6 §6.1 for the CTR mode-specific detail. Different contexts so brief overlap is acceptable.

✂️ **Cut:** Ch.6 §6.1 re-explains AES at block-cipher level (rounds, SubBytes, etc.) even though the reader has already seen Ch.4. §6.1 can start from "CTR mode requires only the forward cipher" without re-describing AES internals.

---

### R8 — Benchmark reproducibility / run IDs / artifact provenance

| Chapter / Section | Representative Quote |
|---|---|
| Ch.3 §3.1 | "The harness records machine-readable artifacts per run… Core outputs are aggregated into aggregated.json" |
| Ch.8 §8.1 | "make every security-relevant output inspectable in the same evidence trail" |
| Ch.8 §8.4 | "two linked evidence layers: cryptographic correctness / measurement evidence" |
| Ch.9 §9.5 | Reproducible Execution — concrete commands listed |
| Ch.11 §11.1 | "evaluation source is the report benchmark campaign" |

✅ **Keep:** Ch.3 §3.1 (methodology), Ch.9 §9.5 (concrete commands), Ch.8 §8.4 (dual-evidence principle — state once here).

✂️ **Cut:** Ch.11 §11.1 repeats the one-trial policy already stated in Ch.3. Ch.8 §8.1 and §8.4 contain overlapping traceability statements — merge into one paragraph.

---

### R9 — Claim that circuit-level intuition does not transfer to zkVM

| Chapter / Section | Representative Quote |
|---|---|
| Ch.1 §1.2 | "naive performance transfer between models often fails" |
| Ch.2 §2.5 | "constraint-level efficiency does not automatically imply zkVM efficiency" |
| Ch.5 §5.3 | "those gains do not automatically transfer to VM-trace cost in this implementation stack" |
| Ch.5 §5.4 | "VM-vs-circuit optimization gap" |
| Ch.11 §11.8 | "VM-level software costs can dominate and reverse expected rankings" |

✅ **Keep:** Ch.2 §2.5 (background argument) and Ch.5 §5.3–5.4 (evidence-backed analysis). One closing sentence in Ch.11 §11.8 is fine.

✂️ **Cut:** Ch.1 §1.2 introduces this argument before the reader has any background to evaluate it. Reduce to one sentence and develop fully only in Ch.2 and Ch.5.

---

## LOW Severity — Minor Tidy-Up

---

### R10 — LowMC's matrix-path software overhead explanation

| Chapter / Section | Representative Quote |
|---|---|
| Ch.4 §4.2 | "relies heavily on binary linear layers, especially matrix–vector products over GF(2)" |
| Ch.4 §4.5 | "LowMC's matrix/bit-transformation-heavy structure corresponds to very high full-cipher proving cost" |
| Ch.5 §5.4 | "matrix-heavy operations…create substantial trace and runtime pressure" |

✅ **Keep:** Ch.4 §4.2 (definitional) and Ch.5 §5.4 (root-cause analysis).

✂️ **Cut:** Ch.4 §4.5 "Matches expectation" paragraph repeats the conclusion from Table 4.1 one page after it is drawn. Fold into a single concluding sentence in §4.3 or §4.4.

---

### R11 — HMAC-SHA256 / RFC 2104 / FIPS standards citation block

| Chapter / Section | Representative Quote |
|---|---|
| Ch.2 §2.6 | "HMAC-SHA256 as standardized by RFC 2104, FIPS 198-1, and FIPS 180-4" |
| Ch.6 §6.1 | "HMAC-SHA256 (truncated to 192 bits)…" |
| Ch.7 §7.1 | "RFC 2104, and FIPS 198-1" |

✅ **Keep:** Ch.2 §2.6 as the canonical reference.

✂️ **Cut:** The full RFC/FIPS citation block is copied verbatim into Ch.7 §7.1. Replace with "as described in §2.6" — one cross-reference is enough.

---

### R12 — Authenticated pipeline target split (aes-ctr vs aes-ctr-hmac)

| Chapter / Section | Representative Quote |
|---|---|
| Ch.6 §6.1 | "two explicit repository targets: aes-ctr and aes-ctr-hmac" |
| Ch.8 §8.5 | "aes-ctr-1blk, aes-ctr-4blk… / aes-ctr-hmac-1blk…" |
| Ch.9 §9.1 | "final authenticated pipeline split: aes-ctr (CTR-only) and aes-ctr-hmac" |

✅ **Keep:** Ch.8 §8.5 (design rationale and naming) and Ch.9 §9.1 (implementation mapping).

✂️ **Cut:** Ch.6 §6.1 introduces the split before the design chapter. Replace with a forward reference: "detailed in Chapter 8."

---

### R13 — Future work: commit-locked multi-trial N≥5 rerun

| Chapter / Section | Representative Quote |
|---|---|
| Ch.3 §3.4 | "For final production-grade benchmarking, the intended policy is N≥5" |
| Ch.11 §11.9 | "A stronger evaluation would repeat every headline target with N≥5" |
| Ch.11 §11.11 | "rerun the full benchmark campaign with N≥5 under explicit commit-lock capture" |
| Ch.14 §14.1 | "Run a fully commit-locked multi-trial campaign (N≥5)" |

✅ **Keep:** Ch.14 §14.1 (the correct home for the roadmap) and a brief pointer in Ch.12 §12.1.

✂️ **Cut:** Ch.3 §3.4 can acknowledge N=1 without proposing future work. Ch.11 §11.9 and §11.11 both repeat the same proposal — consolidate into one sentence pointing to Ch.14.

---

## Chapter-by-Chapter Cut Summary

| Chapter | What to Cut or Shorten |
|---|---|
| Abstract | Keep 34.85× and 25.59% figures; remove the detailed HMAC pipeline description — it duplicates Ch.8. |
| Ch.1 §1.2 | Reduce the "naive performance transfer" argument to one sentence; develop fully in Ch.2 §2.5. |
| Ch.1 §1.6 Key Results | **Delete the entire section.** It is a verbatim preview of Ch.5 results. Replace with two sentences pointing forward to Ch.5 and Ch.11. |
| Ch.1 §1.7 Limitations | Keep to one sentence; the full single-trial caveat belongs in Ch.3 §3.4 and Ch.12. |
| Ch.2 §2.6 | Keep — this is the canonical RFC/FIPS citation block. All later chapters should cross-reference here. |
| Ch.3 §3.1 | Remove the re-explanation of host/guest split (already in Ch.2 §2.2). Keep experimental-environment specifics only. |
| Ch.3 §3.4 | Keep the N=1 rationale. Remove the "intended policy N≥5" forward-looking paragraph — it belongs in Ch.14. |
| Ch.4 §4.5 | Fold the "Matches expectation" paragraph into a single concluding sentence in §4.3. |
| Ch.5 §5.3 | Keep — earns its place by contextualising against prior literature. |
| Ch.6 §6.1 | Remove: AES internal round description (→ Ch.4 §4.2), RFC/FIPS citations (→ Ch.2 §2.6), target-split explanation (→ Ch.8). Replace each with a forward reference. |
| Ch.7 §7.2 | Trim to bullet-point threats only; remove the layer-resolution discussion (belongs in §7.3). |
| Ch.7 §7.1 | Remove the duplicated RFC/FIPS block; cross-reference Ch.2 §2.6. |
| Ch.8 §8.1 / §8.4 | Merge the two overlapping "auditable evidence trail" statements into one paragraph. |
| Ch.9 §9.2–9.3 | Remove the repeated verify-before-decrypt explanation (already in Ch.8 §8.3). One sentence + reference is enough. |
| Ch.11 §11.1 | Remove re-statement of N=1 policy (already in Ch.3 §3.4). Keep the campaign summary only. |
| Ch.11 §11.7 | Replace 34.85× and 25.59% re-statements with "As reported in Table 5.1 and Figure 5.1…" and one concluding sentence. |
| Ch.11 §11.9 / §11.11 | Merge both N≥5 future-work mentions into one sentence pointing to Ch.14. |
| Ch.12 §12.1 | Keep — canonical limitations statement for reproducibility. |
| Ch.14 §14.1 | Keep — the correct home for the N≥5 roadmap. |
| Ch.15 Conclusion | The 517.209 s vs 14.843 s figures are fine. Remove the block-size HMAC percentage range (1.38%–50.15%) — too granular for a conclusion; a qualitative summary suffices. |

---

## Recommended Edit Priority

If you want the biggest impact with the least effort, tackle these three first:

**1. Delete Ch.1 §1.6 (Key Results Preview)**
This entire section reproduces results from Ch.5 and Ch.11 verbatim before the reader has any context. Replace with: *"Headline results are presented in Chapters 5 and 11."*

**2. Consolidate all single-trial caveats into Ch.3 §3.4 and Ch.12 §12.1**
Then in Ch.11, replace every repeated warning with: *"As noted in §3.4, results are single-trial and interpreted directionally where deltas are small."*

**3. Trim Ch.6 §6.1 to decision + justification only**
Remove the re-explanation of AES rounds, the CTR mode walkthrough, and the standards citations — each already has a designated chapter. Replace with forward references.

> These three changes alone eliminate the most visible repetition and tighten the dissertation's narrative arc considerably.
