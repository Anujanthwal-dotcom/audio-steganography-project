# Project Specification: Audio Steganography Rust TUI

**Project Name (Working Title):** `Sonosteg` (or `EchoCrypt TUI`)  
**Target Platform:** Linux / macOS / Windows (Cross-platform CLI/TUI)  
**Language & Ecosystem:** Rust 2021 Edition, Ratatui, Crossterm  

---

## 1. Executive Summary & Objectives

The goal of this project is to build an interactive, high-performance, and secure **Terminal User Interface (TUI)** application in Rust for **Audio Steganography**. The tool enables users to:
1. Browse and select audio files (initially targeting uncompressed PCM formats such as `.wav`) directly from their local filesystem.
2. Embed secret textual messages into carrier audio files using imperceptible bit-level steganography techniques with optional authenticated encryption.
3. Extract hidden messages from stego-audio files seamlessly with integrity and authenticity checks.
4. Visualize audio metadata, capacity limits, and steganographic integrity indicators directly within an intuitive, modern terminal interface.

---

## 2. Theory & Fundamentals of Audio Steganography

### 2.1 What is Audio Steganography?
Steganography (from Greek *steganos*, meaning "hidden", and *graphein*, meaning "writing") is the practice of concealing a secret message, file, or image within another ordinary file (the **carrier** or **cover**).

Unlike **Cryptography**, which conceals the *contents* of a secret communication (making it unreadable, though obvious that encryption is used), **Steganography** conceals the *very existence* of the communication. When combined, an encrypted ciphertext is hidden inside audio, yielding both confidentiality and plausible deniability.

* **Cover Audio:** The original, unaltered audio file.
* **Payload / Message:** The secret plaintext or ciphertext to be hidden.
* **Stego Key (Optional):** A cryptographic key or passphrase used to encrypt and pseudo-randomly distribute bits.
* **Stego Audio:** The resulting audio file carrying the embedded message.

```
[ Secret Text ] ---> [ Optional Compression & Encryption ]
                              |
[ Cover Audio (.wav) ] -------> [ Steganographic Encoder ] ---> [ Stego Audio (.wav) ]
                                          |
                                   (Stego Passphrase)
```

---

### 2.2 Anatomy of Digital Audio (PCM WAV)
Uncompressed audio is commonly stored as **Pulse Code Modulation (PCM)** in RIFF/WAV format:
* **Sample Rate:** Frequency of discrete measurements per second (e.g., $44,100\text{ Hz}$ or $48,000\text{ Hz}$).
* **Bit Depth:** Number of bits allocated to represent the amplitude of each sample (typically 16-bit or 24-bit signed integers: `i16` / `i32`).
* **Channels:** Mono (1 channel) or Stereo (2 channels).

In a 16-bit stereo audio file at $44.1\text{ kHz}$:
* 1 second contains $44,100 \times 2 = 88,200\text{ samples}$.
* Each sample is a signed 16-bit integer ranging from $-32,768$ to $+32,767$.

---

### 2.3 Steganography Techniques Comparison

| Technique | Description | Capacity | Robustness against Compression | Audio Quality Impact | Implementation Complexity |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **LSB (Least Significant Bit)** | Modifies the lowest bit(s) of PCM samples | **High** (~5.5 KB/sec for 1 LSB @ 44.1kHz 16-bit stereo) | Low (Fragile to MP3/AAC compression) | **Imperceptible** (SNR > 80 dB) | Moderate (Optimal for WAV/FLAC) |
| **Phase Coding** | Modulates the initial phase of audio frequency segments | Low | Moderate | Minor phase distortion | High |
| **Spread Spectrum / Parity** | Spreads bits across frequency bands or sample blocks | Very Low | High | Low | Very High |
| **Echo Hiding** | Introduces short delays (echoes) below perceptual threshold | Low | High | Potential audible artifacts | High |

#### Selected Engine: Permuted LSB with Key-Derived PRNG
For our Rust TUI application, we employ **Least Significant Bit (LSB) Substitution** with cryptographic enhancements:
1. **LSB Replacement:** Altering the lowest bit of a 16-bit integer changes amplitude by at most 1 unit out of 65,536 ($\approx 0.0015\%$). The human auditory system cannot detect this variance.
2. **Pseudo-Random Dispersion (Stego Key):** Instead of sequentially writing from sample 0 (which is trivially detected by statistical chi-square tests or visual bit-plane analysis), bits are scattered across sample indices using a ChaCha20-based pseudo-random number generator seeded with a key derived from the user's password (via Argon2id).

---

### 2.4 Payload Framing & Wire Protocol
To extract data reliably without prior knowledge of message length, the embedded bitstream must follow a structured binary header:

| Field | Size | Description |
| :--- | :--- | :--- |
| **Magic Bytes** | 4 bytes | Identifies valid stego content (`[0x53, 0x54, 0x45, 0x47]` $\to$ `"STEG"`) |
| **Protocol Version**| 1 byte | Protocol version (`0x01`) |
| **Flags** | 1 byte | Bit 0: Encrypted (`0` = plaintext, `1` = encrypted)<br>Bit 1: Compressed (`0` = raw, `1` = zstd) |
| **Salt** | 16 bytes | Argon2id salt (if encrypted; zeroed if not) |
| **Nonce / IV** | 12 bytes | ChaCha20-Poly1305 / AES-GCM nonce |
| **Payload Length** | 4 bytes | Big-endian unsigned 32-bit integer indicating payload byte count |
| **Ciphertext/Data** | $N$ bytes | Compressed and/or encrypted message payload |
| **HMAC / Auth Tag** | 16 bytes | Poly1305 authentication tag ensuring integrity |
| **CRC32 Checksum** | 4 bytes | Checksum for rapid corruption detection |

---

## 3. Architecture & Tech Stack

### 3.1 Technology Stack
* **Language:** Rust (Stable)
* **TUI Framework:** `ratatui` (v0.29+)
* **Terminal Backend:** `crossterm` (Raw mode, event polling, mouse support)
* **Audio Parsing:** `hound` (Zero-allocation, pure-Rust WAV reader and writer)
* **Cryptography:**
  * `argon2`: Password-based key derivation function (KDF)
  * `chacha20poly1305` or `aes-gcm`: AEAD authenticated encryption
  * `rand_chacha`: Cryptographically secure deterministic sample index permutation
* **Compression:** `zstd` or `flate2` (reduces required audio capacity)
* **Integrity:** `crc32fast`

---

### 3.2 Crate / Module Layout

```
src/
├── main.rs                   # Entrypoint & panic-safe terminal lifecycle
├── app.rs                    # Application state machine & event dispatcher
├── config.rs                 # Theme & user preferences
├── audio/
│   ├── mod.rs                # Audio handling abstractions
│   ├── wav.rs                # Hound wrapper: reading/writing WAV samples
│   └── metadata.rs           # Sample rate, bit depth, channel count, duration
├── stego/
│   ├── mod.rs                # Stego facade
│   ├── lsb.rs                # Core LSB encoding/decoding bit engine
│   ├── frame.rs              # Header framing, magic bytes, serialization
│   ├── capacity.rs           # Capacity calculation & estimation
│   └── disperser.rs          # PRNG-based Fisher-Yates sample permutation
├── crypto/
│   ├── mod.rs
│   ├── kdf.rs                # Argon2id passphrase derivation
│   └── cipher.rs             # ChaCha20-Poly1305 encryption & decryption
└── ui/
    ├── mod.rs                # Root UI rendering loop
    ├── theme.rs              # Color scheme, styles, symbols
    ├── components/
    │   ├── header.rs         # Top application status bar
    │   ├── footer.rs         # Bottom keybinding shortcuts helper
    │   ├── file_picker.rs    # Interactive filesystem browser
    │   ├── text_input.rs     # Multi-line text editor & secret masking
    │   ├── audio_info.rs     # Audio waveform preview & capacity gauge
    │   └── dialogs.rs        # Popups for errors, success, confirmations
    └── views/
        ├── embed_view.rs     # Embed / Hide workflow screen
        ├── extract_view.rs   # Extract / Reveal workflow screen
        └── inspector_view.rs # Audio diagnostics and stego inspection
```

---

## 4. TUI Design System & User Interface

### 4.1 Design Principles
1. **High Information Density with Visual Clarity:** Clean borders, contextual focus indicators, and minimal visual noise.
2. **Deterministic Keyboard Navigation:** Comprehensive vim-style bindings (`h`/`j`/`k`/`l`, `Tab`, `Esc`, `Enter`) with on-screen hint reminders.
3. **Responsive Layout:** Dynamically adapts to small terminals (minimum target: $80 \times 24$) up to ultra-wide displays.
4. **Immediate Feedback & Safety:** Real-time capacity warnings, progress spinners for audio rendering, and explicit error popups.

### 4.2 Visual Palette & Styling

```
+--------------------------------------------------------------+
| Palette Role         | Color Name       | Hex / ANSI         |
+--------------------------------------------------------------+
| Background           | Dark Slate       | #12141A / Default  |
| Surface / Panels     | Charcoal         | #1E222D / DarkGray |
| Primary Accent       | Electric Cyan    | #00E5FF / Cyan     |
| Secondary Accent     | Neon Violet      | #BD93F9 / Magenta  |
| Success / Safe       | Mint Green       | #50FA7B / Green    |
| Warning / Capacity   | Warm Amber       | #FFB86C / Yellow   |
| Danger / Error       | Crimson          | #FF5555 / Red      |
| Text Normal          | Pure / Soft White| #F8F8F2 / White    |
| Text Muted           | Medium Gray      | #6272A4 / Gray     |
+--------------------------------------------------------------+
```

### 4.3 View Layouts

#### Screen 1: Embed Mode (Hiding Text in Audio)
```
┌─ SONOSTEG v0.1.0 ─────────────────────────────────────────────── [Mode: EMBED] ─┐
│ [1] File Selection    │ [2] Secret Payload            │ [3] Audio Inspection     │
├───────────────────────┴───────────────────────────────┴──────────────────────────┤
│ ┌─ Audio File Browser ────────────────┐ ┌─ Secret Message Editor ──────────────┐ │
│ │ /home/user/Music/project/           │ │ Enter message to embed:              │ │
│ │  📁 ..                              │ │                                      │ │
│ │  📁 recordings/                     │ │ "The rendezvous point has moved to   │ │
│ │  🎵 vocal_track.wav  (44.1k, 16bit) │ │  Sector 7 at 0400Z."                 │ │
│ │  🎵 drums.wav        (48.0k, 24bit) │ │                                      │ │
│ │  🎵 bassline.wav     (44.1k, 16bit) │ │                                      │ │
│ │                                     │ ├──────────────────────────────────────┤ │
│ │ Selected: vocal_track.wav           │ │ Passphrase (optional): [ ********** ]│ │
│ └─────────────────────────────────────┘ └──────────────────────────────────────┘ │
│ ┌─ Audio Capacity & Embedding Stats ───────────────────────────────────────────┐ │
│ │ Total Audio Samples: 1,323,000  | Duration: 00:30.00  | Channels: 2 (Stereo) │ │
│ │ Max Embedding Capacity: 330.75 KB                                             │ │
│ │ Message Payload Size:   62 Bytes (0.01% of total capacity)                   │ │
│ │ Capacity: [■■░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0.01% Safe │ │
│ └──────────────────────────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────────────┤
│ [Tab] Switch Focus │ [Enter] Select/Embed │ [E] Extract Mode │ [Ctrl+S] Save │ [Q] Quit │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### Screen 2: Extract Mode (Revealing Text from Audio)
```
┌─ SONOSTEG v0.1.0 ───────────────────────────────────────────── [Mode: EXTRACT] ─┐
│ [1] Carrier Selection                 │ [2] Decrypted Output                     │
├───────────────────────────────────────┴──────────────────────────────────────────┤
│ ┌─ Select Stego Audio File ───────────┐ ┌─ Extracted Secret Message ───────────┐ │
│ │ 📁 ..                               │ │ [DECRYPTION SUCCESSFUL - CRC VALID]  │ │
│ │ 🎵 vocal_track_stego.wav            │ │                                      │ │
│ │                                     │ │ "The rendezvous point has moved to   │ │
│ │ File Size: 5.2 MB                   │ │  Sector 7 at 0400Z."                 │ │
│ │ Detected Stego Magic: "STEG" (v1)   │ │                                      │ │
│ │ Encryption: ChaCha20-Poly1305       │ │                                      │ │
│ ├─────────────────────────────────────┤ │                                      │ │
│ │ Passphrase: [ ********** ]          │ │                                      │ │
│ │ [ Unlock & Extract Payload ]        │ └──────────────────────────────────────┘ │
│ └─────────────────────────────────────┘ ┌─ Actions ────────────────────────────┐ │
│                                         │ [C] Copy to Clipboard  [S] Save File │ │
│                                         └──────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────────────┤
│ [Tab] Switch Focus │ [Enter] Extract │ [H] Embed Mode │ [Esc] Clear │ [Q] Quit   │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. End-to-End Operational Workflows

### 5.1 Workflow A: Embedding Secret Text

```
[ User selects .wav ]
         │
         ▼
[ Read WAV Headers ]
         │
         ├─ Verify format: PCM 16-bit or 24-bit
         └─ Compute total sample count & max capacity
         │
         ▼
[ User types message & optional password ]
         │
         ▼
[ Capacity Check ]
         │
         ├─ Does payload + 44-byte frame header fit in carrier?
         │    ├─ NO  ──> Display error banner (Red dialog, block execution)
         │    └─ YES ──> Continue
         │
         ▼
[ Preprocessing & Encryption ]
         │
         ├─ Optional: Compress payload with Zstandard
         └─ If password present:
               ├─ Derive 256-bit key using Argon2id with random 16-byte salt
               └─ Encrypt payload via ChaCha20-Poly1305 with random 12-byte nonce
         │
         ▼
[ Construct Frame ]
         │
         └─ [ MAGIC (4B) | VERSION (1B) | FLAGS (1B) | SALT (16B) | NONCE (12B) | LEN (4B) | PAYLOAD | TAG (16B) | CRC (4B) ]
         │
         ▼
[ Sample Dispersion & LSB Injection ]
         │
         ├─ Seed ChaCha20 PRNG with KDF key (or fixed seed if unencrypted)
         ├─ Generate non-repeating sequence of sample indices
         └─ Invert or set bit 0 of each selected sample to match frame bits
         │
         ▼
[ Write Stego WAV File ]
         │
         ├─ Save as `<original>_stego.wav` (preserve all RIFF metadata chunks)
         └─ Display confirmation & summary stats in TUI
```

---

### 5.2 Workflow B: Extracting Secret Text

```
[ User selects stego .wav file ]
         │
         ▼
[ Scan Sample Indices for Header ]
         │
         ├─ Read first 32 bits from initial designated indices
         └─ Check for Magic Bytes `0x53 0x54 0x45 0x47` ("STEG")
               ├─ Not Found ──> Display "No stego signature found" notification
               └─ Found     ──> Continue
         │
         ▼
[ Parse Header ]
         │
         ├─ Extract Version, Flags, Salt, Nonce, and Payload Length
         └─ If encrypted flag is set: Prompt user for Passphrase
         │
         ▼
[ Extract Remaining Bitstream ]
         │
         ├─ Read remaining $N \times 8$ bits from PRNG-dispersed sample indices
         └─ Reassemble into raw bytes
         │
         ▼
[ Integrity Verification & Decryption ]
         │
         ├─ If encrypted:
         │     ├─ Derive key using extracted Salt & user Passphrase
         │     └─ Decrypt & verify Poly1305 MAC tag
         │           ├─ Invalid key ──> Display "Incorrect Passphrase / Corrupted"
         │           └─ Valid       ──> Continue
         ├─ Verify CRC32 checksum
         └─ If compressed: Decompress with Zstandard
         │
         ▼
[ Display Extracted Plaintext ]
         │
         └─ Render in scrollable TUI text viewer with options to copy or export
```

---

## 6. Edge Cases & Resilience Strategy

1. **Non-PCM Audio Files (MP3, AAC, OGG):**
   * *Issue:* Lossy compression modifies frequency coefficients and destroys LSB data.
   * *Resolution:* The file browser filters for `.wav` (and optionally uncompressed `.flac`). Non-supported formats display an explanatory warning.
2. **Insufficient Audio Capacity:**
   * *Issue:* The user attempts to embed a message larger than available samples.
   * *Resolution:* The TUI dynamically calculates and displays a capacity progress bar in real time as the user types, disabling the "Embed" button when overflow occurs.
3. **Corrupted or Manipulated Stego Audio:**
   * *Issue:* Audio editing software (normalization, EQ) alters samples.
   * *Resolution:* CRC32 checksum and AEAD Poly1305 authentication tags immediately flag corruption rather than outputting garbage characters.
4. **Terminal Resizing:**
   * *Issue:* Terminal window resized below minimum dimensions.
   * *Resolution:* Gracefully render a fallback warning: `"Terminal window too small (minimum: 80x24)"`.
5. **Clean Terminal Restoration:**
   * *Issue:* App panics or crashes leaving the user's terminal in broken raw mode.
   * *Resolution:* Custom panic hook using `std::panic::set_hook` to guarantee terminal restoration (`disable_raw_mode()`, `LeaveAlternateScreen`, `show_cursor()`).

---

## 7. Implementation Roadmap

- [x] **Phase 1: Architecture & Specification Document** (`specification.md`)
- [x] **Phase 2: Cargo Scaffolding & Dependencies Setup**
  - Initialize `Cargo.toml` with `ratatui`, `crossterm`, `hound`, `chacha20poly1305`, `argon2`, etc.
- [x] **Phase 3: Core Steganography & Audio Engine**
  - Implement WAV reader/writer wrapper via `hound`.
  - Implement bit-packing, framing, LSB replacement, and PRNG sample dispersion.
  - Unit tests verifying 100% roundtrip accuracy (Embed $\to$ Extract).
- [x] **Phase 4: Cryptography & Integrity Subsystem**
  - Argon2id KDF and ChaCha20-Poly1305 AEAD integration.
  - CRC32 verification.
- [x] **Phase 5: TUI Interface & Navigation**
  - Filesystem browser component for audio selection.
  - Message input component and live capacity indicator.
  - Embed and Extract views with responsive layouts.
- [x] **Phase 6: Polish, Testing & Validation**
  - Audible transparency tests (SNR calculation).
  - Panic-safe terminal handling and error notifications.
