use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::Deserialize;

// Type alias for the top-level array structure
pub type ExifOutput = Vec<ExifData>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExifData {
    // --- Date Fields ---
    #[serde(deserialize_with = "crate::parse::datetime::parse_fixed_datetime")]
    pub file_modify_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_fixed_datetime")]
    pub file_access_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_fixed_datetime")]
    pub file_create_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime")]
    pub modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime")]
    pub create_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime")]
    pub date_time_original: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime_with_subsec")]
    pub sub_sec_create_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime_with_subsec")]
    pub sub_sec_date_time_original: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime_with_subsec")]
    pub sub_sec_modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse::datetime::parse_naive_datetime")]
    pub profile_date_time: Option<NaiveDateTime>,

    // --- File Metadata ---
    pub source_file: Option<String>,
    pub exif_tool_version: Option<f64>,
    pub file_name: Option<String>,
    pub directory: Option<String>,
    pub file_size: Option<String>,
    pub zone_identifier: Option<String>,

    pub file_permissions: Option<String>,
    pub file_type: Option<String>,
    pub file_type_extension: Option<String>,
    #[serde(alias = "MIMEType")]
    pub mime_type: Option<String>,

    // --- EXIF/Image Metadata ---
    #[serde(alias = "JFIFVersion")]
    pub jfif_version: Option<f64>,
    pub exif_byte_order: Option<String>,

    #[serde(alias = "GPSAltitudeRef")]
    pub gps_altitude_ref: Option<String>,
    pub model: Option<String>, // Camera Model
    #[serde(alias = "YCbCrPositioning")]
    pub y_cb_cr_positioning: Option<String>,
    pub resolution_unit: Option<String>,
    #[serde(alias = "YResolution")]
    pub y_resolution: Option<f64>, // Use f64 for potential float values
    pub orientation: Option<String>,
    pub software: Option<String>,
    pub color_space: Option<String>,
    #[serde(alias = "FNumber")]
    pub f_number: Option<f64>,
    pub subject_distance_range: Option<String>,
    pub focal_length: Option<String>, // "4.7 mm", keep as string due to unit
    pub aperture_value: Option<f64>,
    pub exposure_mode: Option<String>,
    pub sub_sec_time_digitized: Option<u32>,
    pub exif_image_height: Option<u32>,
    pub focal_length_in_35mm_format: Option<String>, // "0 mm", string due to unit
    pub scene_capture_type: Option<String>,
    pub scene_type: Option<String>,
    pub sub_sec_time_original: Option<u32>,
    pub exposure_program: Option<String>,
    pub white_balance: Option<String>,
    pub exif_image_width: Option<u32>,
    pub sub_sec_time: Option<u32>, // Duplicate of others? ExifTool redundancy.
    pub shutter_speed_value: Option<f64>, // This seems like an APEX value or similar
    pub metering_mode: Option<String>,
    pub components_configuration: Option<String>,
    pub subject_distance: Option<String>, // "1.15 m", string due to unit
    pub exif_version: Option<String>,
    pub flash: Option<String>,
    pub interop_index: Option<String>,
    pub interop_version: Option<String>,
    pub exposure_compensation: Option<f64>, // Could be fractional
    pub brightness_value: Option<f64>,
    #[serde(alias = "ISO")]
    pub iso: Option<u32>,
    pub sensing_method: Option<String>,
    pub flashpix_version: Option<String>,
    pub exposure_time: Option<String>, // "1/30", keep as string (fraction)
    #[serde(alias = "XResolution")]
    pub x_resolution: Option<f64>,
    pub make: Option<String>, // Camera Make
    pub thumbnail_length: Option<u32>,
    pub thumbnail_offset: Option<u32>,
    pub compression: Option<String>, // Thumbnail compression

    // --- ICC Profile Fields ---
    pub profile_cmm_type: Option<String>,
    pub profile_version: Option<String>,
    pub profile_class: Option<String>,
    pub color_space_data: Option<String>,
    pub profile_connection_space: Option<String>,

    pub profile_file_signature: Option<String>,
    pub primary_platform: Option<String>,
    #[serde(alias = "CMMFlags")]
    pub cmm_flags: Option<String>,
    pub device_manufacturer: Option<String>, // Profile device, distinct from camera Make
    pub device_model: Option<String>, // Profile device, distinct from camera Model
    pub device_attributes: Option<String>,
    pub rendering_intent: Option<String>,
    pub connection_space_illuminant: Option<String>, // Space separated numbers, keep as String
    pub profile_creator: Option<String>,
    #[serde(alias = "ProfileID")]
    pub profile_id: Option<String>,
    pub profile_description: Option<String>,
    pub blue_matrix_column: Option<String>, // Space separated numbers, keep as String

    // Binary data placeholders - keep as strings
    pub blue_trc: Option<String>,
    pub green_trc: Option<String>,
    pub red_trc: Option<String>,
    pub thumbnail_image: Option<String>,

    pub device_model_desc: Option<String>,
    pub green_matrix_column: Option<String>, // Space separated numbers, keep as String
    pub luminance: Option<String>, // Space separated numbers, keep as String
    pub measurement_observer: Option<String>,
    pub measurement_backing: Option<String>, // Space separated numbers, keep as String
    pub measurement_geometry: Option<String>,
    pub measurement_flare: Option<String>, // "0%", keep as string due to unit
    pub measurement_illuminant: Option<String>,
    pub media_black_point: Option<String>, // Space separated numbers, keep as String
    pub red_matrix_column: Option<String>, // Space separated numbers, keep as String
    pub technology: Option<String>,
    pub viewing_cond_desc: Option<String>,
    pub media_white_point: Option<String>, // Space separated numbers, keep as String
    pub profile_copyright: Option<String>,
    pub chromatic_adaptation: Option<String>, // Space separated numbers, keep as String

    // --- Image Attributes ---
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub encoding_process: Option<String>,
    pub bits_per_sample: Option<u8>,
    pub color_components: Option<u8>,
    #[serde(alias = "YCbCrSubSampling")]
    pub y_cb_cr_sub_sampling: Option<String>,

    // --- Composite Fields (Often derived by ExifTool) ---
    pub aperture: Option<f64>, // Often same as ApertureValue
    pub image_size: Option<String>, // "2688x1512"
    pub megapixels: Option<f64>,
    pub shutter_speed: Option<String>, // "1/30", Often same as ExposureTime
    #[serde(alias = "FocalLength35efl")]
    pub focal_length_35_efl: Option<String>, // String due to unit, distinct from FocalLength
    pub light_value: Option<f64>,

    // If you encounter fields not listed here, add them following the same pattern.
    // Use Option<T> for robustness against missing fields.
    // Use #[serde(alias = "JsonFieldName")] if the JSON field name doesn't match
    // the snake_case version of the Rust field name (though rename_all handles most).
}