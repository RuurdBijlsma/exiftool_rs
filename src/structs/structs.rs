use crate::parse_fn::binary::BinaryDataField;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::Deserialize;

// Type alias for the top-level array structure
pub type ExifOutput = Vec<ExifData>;
// TODO make structs:
// * base file struct with file tags
// * image struct for comon image tags
// * video struct for common video tags
// * jpg/png/xmp/quicktime/matroska/gif structs
// * make composed structs for common use that mix together base+image+video+jpg+png+quicktime+etc....
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ExifData {
    // --- Date Fields ---
    #[serde(deserialize_with = "crate::parse_fn::datetime::fixed")]
    pub file_modify_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::fixed")]
    pub file_access_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::fixed")]
    pub file_create_date: Option<DateTime<FixedOffset>>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub create_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
    pub date_time_original: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive_with_subsec")]
    pub sub_sec_create_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive_with_subsec")]
    pub sub_sec_date_time_original: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive_with_subsec")]
    pub sub_sec_modify_date: Option<NaiveDateTime>,

    #[serde(deserialize_with = "crate::parse_fn::datetime::naive")]
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
    pub y_resolution: Option<f64>,
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
    pub sub_sec_time: Option<u32>,
    pub shutter_speed_value: Option<f64>,
    pub metering_mode: Option<String>,
    pub components_configuration: Option<String>,
    pub subject_distance: Option<String>, // "1.15 m", string due to unit
    pub exif_version: Option<String>,
    pub flash: Option<String>,
    pub interop_index: Option<String>,
    pub interop_version: Option<String>,
    pub exposure_compensation: Option<f64>,
    pub brightness_value: Option<f64>,
    #[serde(alias = "ISO")]
    pub iso: Option<u32>,
    pub sensing_method: Option<String>,
    pub flashpix_version: Option<String>,
    pub exposure_time: Option<String>,
    #[serde(alias = "XResolution")]
    pub x_resolution: Option<f64>,
    pub make: Option<String>, // Camera Make
    pub thumbnail_length: Option<u32>,
    pub thumbnail_offset: Option<u32>,
    pub compression: Option<String>,

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
    pub device_model: Option<String>,        // Profile device, distinct from camera Model
    pub device_attributes: Option<String>,
    pub rendering_intent: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub connection_space_illuminant: Option<Vec<f64>>,
    pub profile_creator: Option<String>,
    #[serde(alias = "ProfileID")]
    pub profile_id: Option<String>,
    pub profile_description: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub blue_matrix_column: Option<Vec<f64>>,

    // Binary data
    #[serde(
        alias = "BlueTRC",
        deserialize_with = "crate::parse_fn::binary::binary"
    )]
    pub blue_trc: Option<BinaryDataField>,
    #[serde(
        alias = "GreenTRC",
        deserialize_with = "crate::parse_fn::binary::binary"
    )]
    pub green_trc: Option<BinaryDataField>,
    #[serde(alias = "RedTRC", deserialize_with = "crate::parse_fn::binary::binary")]
    pub red_trc: Option<BinaryDataField>,
    #[serde(deserialize_with = "crate::parse_fn::binary::binary")]
    pub thumbnail_image: Option<BinaryDataField>,

    pub device_model_desc: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub green_matrix_column: Option<Vec<f64>>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub luminance: Option<Vec<f64>>,
    pub measurement_observer: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub measurement_backing: Option<Vec<f64>>,
    pub measurement_geometry: Option<String>,
    pub measurement_flare: Option<String>, // "0%", keep as string due to unit
    pub measurement_illuminant: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub media_black_point: Option<Vec<f64>>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub red_matrix_column: Option<Vec<f64>>,
    pub technology: Option<String>,
    pub viewing_cond_desc: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub media_white_point: Option<Vec<f64>>,
    pub profile_copyright: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats")]
    pub chromatic_adaptation: Option<Vec<f64>>,

    // --- Image Attributes ---
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub encoding_process: Option<String>,
    pub bits_per_sample: Option<u8>,
    pub color_components: Option<u8>,
    #[serde(alias = "YCbCrSubSampling")]
    pub y_cb_cr_sub_sampling: Option<String>,

    // --- Composite Fields (Often derived by ExifTool) ---
    pub aperture: Option<f64>,      // Often same as ApertureValue
    pub image_size: Option<String>, // "2688x1512"
    pub megapixels: Option<f64>,
    pub shutter_speed: Option<String>, // "1/30", Often same as ExposureTime
    #[serde(alias = "FocalLength35efl")]
    pub focal_length_35_efl: Option<String>, // String due to unit, distinct from FocalLength
    pub light_value: Option<f64>,
}
