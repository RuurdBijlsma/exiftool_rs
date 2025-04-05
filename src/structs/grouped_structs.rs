use crate::parse_fn::binary::BinaryDataField;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ExifFile {
    pub source_file: String,
    // TODO: Audio
    // TODO: Author
    #[serde(rename = "Camera")]
    pub camera: Option<CameraData>,
    // TODO: Device
    // TODO: Document
    #[serde(rename = "ExifTool")]
    pub exif_tool: Option<ExifToolData>,
    #[serde(rename = "Image")]
    pub image: Option<ImageData>,
    #[serde(rename = "Location")]
    pub location: Option<LocationData>,
    #[serde(rename = "Other")]
    pub other: Option<OtherData>,
    #[serde(rename = "Preview")]
    pub preview: Option<PreviewData>,
    // TODO: Printing
    #[serde(rename = "Time")]
    pub time: Option<TimeData>,
    // TODO: Unknown
    // TODO: Video
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct CameraData {
    pub device_model_desc: Option<String>,
    pub exposure_mode: Option<String>,
    pub exposure_program: Option<String>,
    pub flash: Option<String>,
    pub focal_length: Option<String>, // e.g., "4.7 mm"
    #[serde(rename = "FocalLength35efl")]
    pub focal_length_35_efl: Option<String>, // e.g., "4.7 mm"
    #[serde(rename = "FocalLengthIn35mmFormat")]
    pub focal_length_in_35mm_format: Option<String>, // e.g., "0 mm"
    pub make: Option<String>,
    pub metering_mode: Option<String>,
    pub model: Option<String>,
    pub scene_capture_type: Option<String>,
    pub sensing_method: Option<String>,
    pub subject_distance: Option<String>, // e.g., "1.15 m"
    pub subject_distance_range: Option<String>,
    pub white_balance: Option<String>,
    // --- New fields from MOV ---
    // Note: Some fields like ExposureTime, FNumber, ISO also exist in ImageData
    // They are kept separate here as they appear under the "Camera" group specifically.
    pub exposure_time: Option<String>, // e.g., "1/38"
    #[serde(rename = "FNumber")]
    pub f_number: Option<f64>,
    pub exposure_compensation: Option<f64>, // Or f64 if it can be fractional
    #[serde(rename = "ISO")]
    pub iso: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ExifToolData {
    pub exif_tool_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ImageData {
    pub aperture: Option<f64>,
    pub aperture_value: Option<f64>,
    pub bits_per_sample: Option<u32>,
    pub blue_matrix_column: Option<String>,

    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub blue_trc: Option<BinaryDataField>,

    pub brightness_value: Option<f64>,
    #[serde(rename = "CMMFlags")]
    pub cmm_flags: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub chromatic_adaptation: Option<Vec<f64>>,
    pub color_components: Option<u32>,
    pub color_space: Option<String>,
    pub color_space_data: Option<String>,
    pub components_configuration: Option<String>,
    pub compression: Option<String>,

    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub connection_space_illuminant: Option<Vec<f64>>,

    pub device_attributes: Option<String>,
    pub device_manufacturer: Option<String>,
    pub device_model: Option<String>,
    pub encoding_process: Option<String>,
    pub exif_byte_order: Option<String>,
    pub exif_image_height: Option<u32>,
    pub exif_image_width: Option<u32>,
    pub exif_version: Option<String>,
    pub exposure_compensation: Option<f64>,
    pub exposure_time: Option<String>, // e.g., "1/30"
    #[serde(rename = "FNumber")]
    pub f_number: Option<f64>,
    pub flashpix_version: Option<String>,
    pub green_matrix_column: Option<String>,

    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub green_trc: Option<BinaryDataField>,

    #[serde(rename = "ISO")]
    pub iso: Option<u32>,
    pub image_height: Option<u32>,
    pub image_size: Option<String>, // e.g., "2688x1512"
    pub image_width: Option<u32>,
    pub interop_index: Option<String>,
    pub interop_version: Option<String>,
    #[serde(rename = "JFIFVersion")]
    pub jfif_version: Option<f64>,
    pub light_value: Option<f64>,
    pub luminance: Option<String>,
    pub measurement_backing: Option<String>,
    pub measurement_flare: Option<String>,
    pub measurement_geometry: Option<String>,
    pub measurement_illuminant: Option<String>,
    pub measurement_observer: Option<String>,
    pub media_black_point: Option<String>, // Could parse into Vec<f64>
    pub media_white_point: Option<String>, // Could parse into Vec<f64>
    pub megapixels: Option<f64>,
    pub orientation: Option<String>,
    pub primary_platform: Option<String>,
    #[serde(rename = "ProfileCMMType")]
    pub profile_cmm_type: Option<String>,
    pub profile_class: Option<String>,
    pub profile_connection_space: Option<String>,
    pub profile_copyright: Option<String>,
    pub profile_creator: Option<String>,
    pub profile_description: Option<String>,
    pub profile_file_signature: Option<String>,
    #[serde(rename = "ProfileID")]
    pub profile_id: Option<String>,
    pub profile_version: Option<String>,
    pub red_matrix_column: Option<String>,

    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub red_trc: Option<BinaryDataField>,

    pub rendering_intent: Option<String>,
    pub resolution_unit: Option<String>,
    pub scene_type: Option<String>,
    pub shutter_speed: Option<String>,
    pub shutter_speed_value: Option<f64>,
    pub software: Option<String>,
    pub technology: Option<String>,
    pub thumbnail_length: Option<u32>,
    pub thumbnail_offset: Option<u32>,
    pub viewing_cond_desc: Option<String>,
    #[serde(rename = "XResolution")]
    pub x_resolution: Option<f64>,
    #[serde(rename = "YCbCrPositioning")]
    pub y_cb_cr_positioning: Option<String>,
    #[serde(rename = "YCbCrSubSampling")]
    pub y_cb_cr_sub_sampling: Option<String>,
    #[serde(rename = "YResolution")]
    pub y_resolution: Option<f64>,

    // --- New fields from MP3 ---
    #[serde(rename = "ID3Size")]
    pub id3_size: Option<u32>,
    pub picture_format: Option<String>, // e.g., "JPG"
    pub picture_type: Option<String>, // e.g., "Other"
    pub picture_description: Option<String>,

    // --- New fields from MOV ---
    pub compressor_id: Option<String>, // e.g., "jpeg"
    pub vendor_id: Option<String>, // e.g., "Pentax"
    pub source_image_width: Option<u32>,
    pub source_image_height: Option<u32>,
    pub compressor_name: Option<String>, // e.g., "Photo - JPEG"
    pub bit_depth: Option<u32>,
    pub alt_tape_name: Option<String>,
    pub album: Option<String>, // Note: Also in Audio, Video, Other groups
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct LocationData {
    #[serde(rename = "GPSAltitudeRef")]
    pub gps_altitude_ref: Option<String>,
    // Add other GPS fields here if they appear in other files
    // e.g., GPSLatitude, GPSLongitude, GPSAltitude etc.
    // pub gps_latitude: Option<f64>, // Needs custom parsing from DMS usually
    // pub gps_longitude: Option<f64>, // Needs custom parsing
    // pub gps_altitude: Option<f64>, // Needs custom parsing (e.g., "123 m")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")] // Handles fields within the Other group
#[allow(dead_code)]
pub struct OtherData {
    pub directory: Option<String>,
    pub file_name: Option<String>,
    pub file_permissions: Option<String>,
    pub file_size: Option<String>, // e.g., "927 kB" - needs parsing if you want bytes
    pub file_type: Option<String>,
    pub file_type_extension: Option<String>,
    #[serde(rename = "MIMEType")]
    pub mime_type: Option<String>,
    pub zone_identifier: Option<String>, // Windows specific?
    pub album: Option<String>,
    pub artist: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")] // Handles fields within the Preview group
#[allow(dead_code)]
pub struct PreviewData {
    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub thumbnail_image: Option<BinaryDataField>,

    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub picture: Option<BinaryDataField>,

    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub cover_art: Option<BinaryDataField>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")] // Handles fields within the Time group
#[allow(dead_code)]
pub struct TimeData {
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub create_date: Option<NaiveDateTime>, // CUSTOM PARSING

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub date_time_original: Option<NaiveDateTime>, // CUSTOM PARSING

    #[serde(deserialize_with = "crate::parse_fn::datetime::with_timezone")]
    pub file_access_date: Option<DateTime<FixedOffset>>, // e.g. "2025:04:05 18:50:45+02:00"
    #[serde(deserialize_with = "crate::parse_fn::datetime::with_timezone")]
    pub file_create_date: Option<DateTime<FixedOffset>>,
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")] // Assuming same format
    pub profile_date_time: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub sub_sec_create_date: Option<NaiveDateTime>, // CUSTOM PARSING

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub sub_sec_date_time_original: Option<NaiveDateTime>, // CUSTOM PARSING

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub sub_sec_modify_date: Option<NaiveDateTime>, // CUSTOM PARSING

    pub sub_sec_time: Option<u32>,
    pub sub_sec_time_digitized: Option<u32>,
    pub sub_sec_time_original: Option<u32>,
    // --- New fields from MOV ---
    // String format like "2005:08:11 14:03:54"
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub track_create_date: Option<NaiveDateTime>,
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub track_modify_date: Option<NaiveDateTime>,
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub media_create_date: Option<NaiveDateTime>,
    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub media_modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::with_timezone")]
    pub metadata_date: Option<DateTime<FixedOffset>>, // Includes timezone offset "2008:09:12 11:17:39-04:00"
    pub content_create_date: Option<i32>, // e.g., 2010 (numeric year)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct AudioData {
    // --- Fields from MP3 ---
    #[serde(rename = "MPEGAudioVersion")]
    pub mpeg_audio_version: Option<u32>,
    pub audio_layer: Option<u32>,
    pub audio_bitrate: Option<String>, // e.g., "128 kbps"
    // SampleRate / AudioSampleRate - using alias
    #[serde(alias = "SampleRate")]
    pub audio_sample_rate: Option<u32>,
    pub channel_mode: Option<String>,
    #[serde(rename = "MSStereo")]
    pub ms_stereo: Option<String>, // e.g., "On"
    pub intensity_stereo: Option<String>, // e.g., "Off"
    pub copyright_flag: Option<bool>,
    pub original_media: Option<bool>,
    pub emphasis: Option<String>,
    pub track: Option<String>,                      // e.g., "1/5"
    pub part_of_set: Option<String>,                // e.g., "1/2"
    pub relative_volume_adjustment: Option<String>, // e.g., "+18.0% Right, +18.0% Left"
    pub title: Option<String>,
    // Grouping, Lyrics, Composer, Album, Genre, Comment also appear in MOV audio

    // --- Fields from MOV ---
    pub balance: Option<f64>,         // Or i32? Assume f64 for flexibility
    pub audio_format: Option<String>, // e.g., "raw "
    pub audio_channels: Option<u32>,
    pub audio_bits_per_sample: Option<u32>,
    // Lyrics also in MP3
    pub lyrics: Option<String>,
    // Artist, Composer, Album, Grouping, Genre, Comment also potentially in MP3 or other groups
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub album: Option<String>,
    pub grouping: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>, // e.g., "1 of 2"
    pub disk_number: Option<String>,  // e.g., "3 of 4"
    pub comment: Option<String>,
    #[serde(rename = "BeatsPerMinute")]
    pub beats_per_minute: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct AuthorData {
    // --- Fields from MP3 ---
    pub artist: Option<String>,

    // --- Fields from MOV ---
    pub creator: Option<String>,
    pub album_artist: Option<String>, // Or AlbumArtist
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct VideoData {
    // --- Fields from MP3 (Minimal) ---
    // Duration also appears in MOV
    pub duration: Option<String>, // e.g., "0.00 s (approx)", "4.97 s"

    // --- Fields from MOV ---
    pub movie_header_version: Option<u32>,
    pub time_scale: Option<u32>,
    pub preferred_rate: Option<f64>, // 1 in example, but could be float
    pub preferred_volume: Option<String>, // e.g., "99.61%"
    pub preview_time: Option<String>, // e.g., "0 s"
    pub preview_duration: Option<String>,
    pub poster_time: Option<String>,
    pub selection_time: Option<String>,
    pub selection_duration: Option<String>,
    pub current_time: Option<String>,
    pub next_track_id: Option<u32>,
    pub track_header_version: Option<u32>,
    pub track_id: Option<u32>,
    pub track_duration: Option<String>,
    pub track_layer: Option<i32>,     // 0 in example
    pub track_volume: Option<String>, // e.g., "0.00%"
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub clean_aperture_dimensions: Option<String>, // e.g., "320x240"
    pub production_aperture_dimensions: Option<String>,
    pub encoded_pixels_dimensions: Option<String>,
    pub graphics_mode: Option<String>, // e.g., "ditherCopy"
    pub op_color: Option<String>,      // e.g., "32768 32768 32768"
    pub video_frame_rate: Option<f64>,
    pub matrix_structure: Option<String>, // e.g., "1 0 0 0 1 0 0 0 1"
    pub media_header_version: Option<u32>,
    pub media_time_scale: Option<u32>,
    pub media_duration: Option<String>,
    pub handler_class: Option<String>, // e.g., "Data Handler"
    pub format: Option<String>,        // e.g., "Digital Camera"
    pub information: Option<String>,
    // Album, Artist, Comment, Composer, Genre also appear elsewhere
    pub album: Option<String>,
    pub artist: Option<String>,
    pub comment: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub handler_type: Option<String>,      // e.g., "Metadata"
    pub handler_vendor_id: Option<String>, // e.g., "Apple"
    pub media_data_size: Option<u64>,
    pub media_data_offset: Option<u64>,
    pub avg_bitrate: Option<String>, // e.g., "0 bps"
    pub rotation: Option<i32>,       // e.g. 0, 90, 180, 270
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct DocumentData {
    // --- Fields from MOV ---
    #[serde(rename = "XMPToolkit")]
    pub xmp_toolkit: Option<String>,
}
