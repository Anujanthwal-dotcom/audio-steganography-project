use hound::{SampleFormat, WavSpec, WavWriter};
use sonosteg::audio::{calculate_snr, WavData};
use sonosteg::stego::{detect_stego_header, embed_message, extract_message};

fn create_test_wav(
    path: &std::path::Path,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    num_samples: usize,
) {
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();
    for i in 0..num_samples {
        // Generate a 440Hz sine wave tone
        let t = i as f64 / sample_rate as f64;
        let amplitude = (i16::MAX / 2) as f64;
        let val = (amplitude * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i32;

        match bits_per_sample {
            16 => writer.write_sample(val as i16).unwrap(),
            24 | 32 => writer.write_sample(val).unwrap(),
            _ => panic!("Unsupported bit depth in test"),
        }
    }
    writer.finalize().unwrap();
}

#[test]
fn test_end_to_end_16bit_encrypted_wav() {
    let temp_dir = std::env::temp_dir();
    let cover_path = temp_dir.join("sonosteg_test_cover_16.wav");
    let stego_path = temp_dir.join("sonosteg_test_stego_16.wav");

    // 16-bit stereo 44.1kHz, 88,200 samples (1 second)
    create_test_wav(&cover_path, 44100, 2, 16, 88200);

    let cover_wav = WavData::read_from_file(&cover_path).unwrap();
    assert_eq!(cover_wav.metadata.channels, 2);
    assert_eq!(cover_wav.metadata.bits_per_sample, 16);
    assert_eq!(cover_wav.metadata.total_samples, 88200);

    let secret_message = "Agent 007 rendezvous at coordinate 34.0522 N, 118.2437 W at 0400Z. Repeating confidential message payload for steganography compression testing. ".repeat(8);
    let password = "TopSecretPassword!2026";

    let mut stego_wav = cover_wav.clone();
    let frame = embed_message(
        &mut stego_wav.samples,
        secret_message.as_bytes(),
        Some(password),
        true,
    )
    .unwrap();

    assert!(frame.header.is_encrypted());
    assert!(frame.header.is_compressed());

    // Calculate SNR
    let snr = calculate_snr(&cover_wav.samples, &stego_wav.samples);
    // Specification section 2.3: "Imperceptible (SNR > 80 dB)"
    assert!(
        snr > 80.0,
        "Expected SNR > 80 dB for 1-LSB stego, got {:.2} dB",
        snr
    );

    // Save stego WAV to disk
    stego_wav.write_to_file(&stego_path).unwrap();

    // Re-read stego WAV from disk
    let loaded_stego = WavData::read_from_file(&stego_path).unwrap();

    // Verify detection
    let header = detect_stego_header(&loaded_stego.samples).unwrap();
    assert_eq!(header.version, 0x01);
    assert!(header.is_encrypted());
    assert!(header.is_compressed());

    // Extract with correct password
    let extracted_bytes = extract_message(&loaded_stego.samples, Some(password)).unwrap();
    let extracted_str = String::from_utf8(extracted_bytes).unwrap();
    assert_eq!(extracted_str, secret_message);

    // Extraction with incorrect password must fail
    assert!(extract_message(&loaded_stego.samples, Some("WrongPassword")).is_err());

    // Clean up temporary files
    let _ = std::fs::remove_file(cover_path);
    let _ = std::fs::remove_file(stego_path);
}

#[test]
fn test_end_to_end_24bit_plaintext_wav() {
    let temp_dir = std::env::temp_dir();
    let cover_path = temp_dir.join("sonosteg_test_cover_24.wav");
    let stego_path = temp_dir.join("sonosteg_test_stego_24.wav");

    // 24-bit mono 48kHz, 48,000 samples (1 second)
    create_test_wav(&cover_path, 48000, 1, 24, 48000);

    let cover_wav = WavData::read_from_file(&cover_path).unwrap();
    assert_eq!(cover_wav.metadata.channels, 1);
    assert_eq!(cover_wav.metadata.bits_per_sample, 24);

    let message = "Plaintext audio steganography payload test.";
    let mut stego_wav = cover_wav.clone();
    let frame = embed_message(&mut stego_wav.samples, message.as_bytes(), None, false).unwrap();

    assert!(!frame.header.is_encrypted());

    let snr = calculate_snr(&cover_wav.samples, &stego_wav.samples);
    assert!(snr > 90.0);

    stego_wav.write_to_file(&stego_path).unwrap();

    let loaded = WavData::read_from_file(&stego_path).unwrap();
    let extracted = extract_message(&loaded.samples, None).unwrap();
    assert_eq!(String::from_utf8(extracted).unwrap(), message);

    let _ = std::fs::remove_file(cover_path);
    let _ = std::fs::remove_file(stego_path);
}
