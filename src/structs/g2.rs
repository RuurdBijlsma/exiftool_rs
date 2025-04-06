use crate::parse_fn::datetime::MaybeDateTime;
use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ExifData {
    // Top level fields that are not objects in the JSON
    pub source_file: Option<String>,

    // Fields corresponding to JSON objects
    pub audio: Option<AudioMetadata>,
    pub author: Option<AuthorMetadata>,
    pub camera: Option<CameraMetadata>,
    pub document: Option<DocumentMetadata>,
    pub exif_tool: Option<ExifToolMetadata>,
    pub image: Option<ImageMetadata>,
    pub location: Option<LocationMetadata>,
    pub other: Option<OtherMetadata>,
    pub preview: Option<PreviewMetadata>,
    pub time: Option<TimeMetadata>,
    pub unknown: Option<UnknownMetadata>, // For the specific "Unknown" block
    pub video: Option<VideoMetadata>,
}

// --- Sub-Structs ---

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct AudioMetadata {
    pub audio_bits_per_sample: Option<u16>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub audio_channels: Option<String>,
    pub audio_format: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub balance: Option<f64>, // Assuming float is possible
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct AuthorMetadata {
    pub author: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct CameraMetadata {
    pub camera_indices: Option<u32>,
    pub cameras: Option<String>,             // URI
    pub circle_of_confusion: Option<String>, // String due to "mm" unit
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub contrast: Option<String>,
    pub depth_map_confidence_uri: Option<String>,
    pub depth_map_depth_uri: Option<String>,
    pub depth_map_far: Option<f64>,
    pub depth_map_focal_table: Option<String>, // Seems like encoded data
    pub depth_map_focal_table_entry_count: Option<u32>,
    pub depth_map_format: Option<String>,
    pub depth_map_item_semantic: Option<String>,
    pub depth_map_measure_type: Option<String>,
    pub depth_map_near: Option<f64>,
    pub depth_map_units: Option<String>,
    pub device_model_desc: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::undef_or_float::float", default)]
    pub digital_zoom_ratio: Option<f64>,
    pub exposure_mode: Option<String>,
    pub exposure_program: Option<String>,
    pub flash: Option<String>,
    pub flash_energy: Option<f64>,    // Assuming float, likely 0
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub focal_length: Option<String>, // String due to "mm" unit
    #[serde(alias = "FocalLength35efl")]
    pub focal_length_35_efl: Option<String>, // Complex string format
    pub focal_length_in_35mm_format: Option<String>, // String due to "mm" unit
    #[serde(alias = "HDRPMakerNote")]
    pub hdrp_maker_note: Option<String>,
    #[serde(alias = "HdrPlusMakernote")]
    pub hdr_plus_makernote: Option<String>,
    pub hyperfocal_distance: Option<String>, // String due to "m" unit
    pub image_item_semantic: Option<String>,
    pub image_item_uri: Option<String>,
    pub imaging_model_distortion: Option<String>, // Encoded?
    pub imaging_model_distortion_count: Option<u32>,
    pub imaging_model_focal_length_x: Option<f64>,
    pub imaging_model_focal_length_y: Option<f64>,
    pub imaging_model_image_height: Option<u32>,
    pub imaging_model_image_width: Option<u32>,
    pub imaging_model_pixel_aspect_ratio: Option<f64>,
    pub imaging_model_principal_point_x: Option<f64>,
    pub imaging_model_principal_point_y: Option<f64>,
    pub imaging_model_skew: Option<f64>,
    #[serde(alias = "LensID")]
    pub lens_id: Option<String>,
    pub light_source: Option<String>,
    pub make: Option<String>,
    pub max_aperture_value: Option<f64>,
    pub metering_mode: Option<String>,
    pub model: Option<String>,
    // MotionPhoto seems boolean-like (1)
    pub motion_photo: Option<u8>, // Or Option<u8> if other values possible
    pub motion_photo_presentation_timestamp_us: Option<u64>, // Microseconds
    pub motion_photo_version: Option<f64>, // Or u32 if always integer
    pub portrait_note: Option<String>, // Encoded?
    pub portrait_relighting_light_pos: Option<String>, // Encoded?
    pub portrait_relighting_rendering_options: Option<String>, // Encoded?
    pub profiles: Option<String>, // URI

    pub relit_input_image_data: Option<String>,
    pub relit_input_image_mime: Option<String>, // e.g., "image/jpeg"
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub saturation: Option<String>,
    #[serde(alias = "ScaleFactor35efl")]
    pub scale_factor_35_efl: Option<f64>,
    pub scene_capture_type: Option<String>,
    pub sensing_method: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub sharpness: Option<String>,

    pub shot_log_data: Option<String>,
    #[serde(alias = "SpecialTypeID")]
    pub special_type_id: Option<String>,
    pub subject_distance: Option<String>, // String due to unit or "inf"
    pub subject_distance_range: Option<String>,
    pub trait_: Option<String>, // "Trait" is a keyword, using trait_
    #[serde(alias = "Type")]
    pub camera_type: Option<String>, // Renamed from Type to avoid conflict
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub white_balance: Option<String>, // Or String if more complex values
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct DocumentMetadata {
    #[serde(alias = "XMPToolkit")]
    pub xmp_toolkit: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ExifToolMetadata {
    pub exif_tool_version: Option<f64>,
    // Although multiple warnings are listed, they come from different files
    // in the combined JSON. A single file usually has one or more related warnings.
    // Representing as Vec<String> might be better if multiple warnings per file are common.
    // Let's stick to Option<String> first as per the single-value-per-struct-field rule.
    // If you commonly get multiple warnings *for one file*, change to Option<Vec<String>>.
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ImageMetadata {
    pub aperture: Option<f64>,
    pub aperture_value: Option<f64>,
    pub bit_depth: Option<u8>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub blue_matrix_column: Option<Vec<f64>>,
    #[serde(alias = "BlueTRC")]
    pub blue_trc: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::undef_or_float::float", default)]
    pub brightness_value: Option<f64>,
    #[serde(alias = "CFAPattern")]
    pub cfa_pattern: Option<String>, // e.g., "[Green,Red][Blue,Green]"
    #[serde(alias = "CMMFlags")]
    pub cmm_flags: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub chromatic_adaptation: Option<Vec<f64>>,
    pub color_components: Option<u8>,
    pub color_space: Option<String>,
    pub color_space_data: Option<String>,
    pub comment: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub components_configuration: Option<String>, // e.g., "Y, Cb, Cr, -"
    pub composite_image: Option<String>, // e.g., "Composite Image Captured While Shooting"
    #[serde(deserialize_with = "crate::parse_fn::undef_or_float::float", default)]
    pub compressed_bits_per_pixel: Option<f64>, // Can be float
    pub compression: Option<String>,     // e.g., "JPEG (old-style)"
    #[serde(alias = "CompressorID")]
    pub compressor_id: Option<String>, // e.g., "avc1"
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub connection_space_illuminant: Option<Vec<f64>>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub creator_tool: Option<String>, // e.g., "Google"
    pub cropped_area_image_height_pixels: Option<u32>,
    pub cropped_area_image_width_pixels: Option<u32>,
    pub cropped_area_left_pixels: Option<u32>,
    pub cropped_area_top_pixels: Option<u32>,
    #[serde(alias = "CurrentIPTCDigest")]
    pub current_iptc_digest: Option<String>, // Hex string
    pub custom_rendered: Option<String>,
    #[serde(alias = "DOF")]
    pub dof: Option<String>, // Depth of Field string, complex format
    pub dependent_image1_entry_number: Option<u32>,
    pub dependent_image2_entry_number: Option<u32>,
    pub device_attributes: Option<String>,
    pub device_manufacturer: Option<String>,
    pub device_model: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::array_or_int::to_array", default)]
    pub directory_item_length: Option<Vec<u64>>,
    #[serde(
        deserialize_with = "crate::parse_fn::string_list::string_list",
        default
    )]
    pub directory_item_mime: Option<Vec<String>>, // e.g. ["image/jpeg", "video/mp4"]
    #[serde(deserialize_with = "crate::parse_fn::array_or_int::to_array", default)]
    pub directory_item_padding: Option<Vec<u64>>, // Nested arrays [[0,0]]
    #[serde(
        deserialize_with = "crate::parse_fn::string_list::string_list",
        default
    )]
    pub directory_item_semantic: Option<Vec<String>>, // e.g. ["Primary", "MotionPhoto"]
    pub encoding_process: Option<String>, // e.g., "Baseline DCT, Huffman coding"
    pub exif_byte_order: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::u32::permissive", default)]
    pub exif_image_height: Option<u32>,
    #[serde(deserialize_with = "crate::parse_fn::u32::permissive", default)]
    pub exif_image_width: Option<u32>,
    pub exif_version: Option<String>, // e.g., "0232"
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub exposure_compensation: Option<String>, // Often 0
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub exposure_index: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub exposure_time: Option<String>, // String to handle fractions like "1/518" or numbers like 1
    #[serde(alias = "FNumber")]
    pub f_number: Option<f64>,
    #[serde(alias = "FOV")]
    pub fov: Option<String>, // String due to "deg" unit
    pub file_source: Option<String>,      // e.g., "Digital Camera"
    pub flashpix_version: Option<String>, // e.g., "0100"
    pub full_pano_height_pixels: Option<u32>,
    pub full_pano_width_pixels: Option<u32>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub green_matrix_column: Option<Vec<f64>>,
    #[serde(alias = "GreenTRC")]
    pub green_trc: Option<String>,
    #[serde(alias = "IPTCDigest")]
    pub iptc_digest: Option<String>, // Hex string (often same as CurrentIPTCDigest)
    #[serde(
        alias = "ISO",
        deserialize_with = "crate::parse_fn::string::string",
        default
    )]
    pub iso: Option<String>, // String to handle "50, 0, 0" and numbers
    pub image_description: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::u32::permissive", default)]
    pub image_height: Option<u32>,
    pub image_size: Option<String>, // e.g., "2688x1512"
    #[serde(alias = "ImageUniqueID")]
    pub image_unique_id: Option<String>, // Hex or alphanumeric ID
    #[serde(deserialize_with = "crate::parse_fn::u32::permissive", default)]
    pub image_width: Option<u32>,
    pub interop_index: Option<String>, // e.g., "R98 - DCF basic file (sRGB)"
    pub interop_version: Option<String>, // e.g., "0100"
    #[serde(alias = "JFIFVersion")]
    pub jfif_version: Option<f64>,
    pub largest_valid_interior_rect_height: Option<u32>,
    pub largest_valid_interior_rect_left: Option<u32>,
    pub largest_valid_interior_rect_top: Option<u32>,
    pub largest_valid_interior_rect_width: Option<u32>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub light_value: Option<f64>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub luminance: Option<Vec<f64>>,
    #[serde(alias = "MPFVersion")]
    pub mpf_version: Option<String>, // e.g., "0100"
    #[serde(alias = "MPImageFlags")]
    pub mp_image_flags: Option<String>, // e.g., "(none)"
    #[serde(alias = "MPImageFormat")]
    pub mp_image_format: Option<String>, // e.g., "JPEG"
    #[serde(alias = "MPImageLength")]
    pub mp_image_length: Option<u32>,
    #[serde(alias = "MPImageStart")]
    pub mp_image_start: Option<u64>, // Can be large offset
    #[serde(alias = "MPImageType")]
    pub mp_image_type: Option<String>, // e.g., "Undefined"
    pub maker_note_unknown_text: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub measurement_backing: Option<Vec<f64>>,
    pub measurement_flare: Option<String>, // String due to "%"
    pub measurement_geometry: Option<String>,
    pub measurement_illuminant: Option<String>,
    pub measurement_observer: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub media_black_point: Option<Vec<f64>>,
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub media_white_point: Option<Vec<f64>>,
    pub megapixels: Option<f64>,
    pub number_of_images: Option<u32>,
    pub orientation: Option<String>,
    pub other_image_length: Option<u32>,
    pub other_image_start: Option<u32>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub pixel_aspect_ratio: Option<String>, // e.g., "65536:65536"
    pub pose_heading_degrees: Option<f64>,
    pub primary_platform: Option<String>,
    pub profile_cmm_type: Option<String>, // Often empty string
    pub profile_class: Option<String>,
    pub profile_connection_space: Option<String>, // e.g., "XYZ "
    pub profile_copyright: Option<String>,
    pub profile_creator: Option<String>,
    pub profile_description: Option<String>,
    pub profile_file_signature: Option<String>, // e.g., "acsp"
    #[serde(
        alias = "ProfileID",
        deserialize_with = "crate::parse_fn::string::string",
        default
    )]
    pub profile_id: Option<String>, // Hex string
    pub profile_version: Option<String>,        // e.g., "2.0.0"
    pub projection_type: Option<String>,        // e.g., "equirectangular"
    #[serde(deserialize_with = "crate::parse_fn::space_sep::floats", default)]
    pub red_matrix_column: Option<Vec<f64>>,
    #[serde(alias = "RedTRC")]
    pub red_trc: Option<String>,
    pub rendering_intent: Option<String>,
    pub resolution_unit: Option<String>,
    pub scene_type: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub shutter_speed: Option<String>, // String to handle fractions like "1/518" or numbers like 1
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub shutter_speed_value: Option<String>, // String to handle fractions like "1/100" or numbers
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub software: Option<String>,
    pub source_image_height: Option<u32>,
    pub source_image_width: Option<u32>,
    pub source_photos_count: Option<u32>,
    pub technology: Option<String>,
    pub thumbnail_length: Option<u32>,
    pub thumbnail_offset: Option<u64>, // Can be large
    #[serde(alias = "UniqueCameraModel")]
    pub unique_camera_model: Option<String>, // Sometimes more specific than Model
    pub use_panorama_viewer: Option<bool>,
    pub user_comment: Option<String>, // Often contains structured text
    pub version: Option<f64>,         // Usually 1.0 for UserComment version? Check context.
    pub viewing_cond_desc: Option<String>,
    #[serde(
        alias = "XResolution",
        deserialize_with = "crate::parse_fn::undef_or_float::float",
        default
    )]
    pub x_resolution: Option<f64>,
    #[serde(alias = "YCbCrPositioning")]
    pub y_cb_cr_positioning: Option<String>,
    #[serde(alias = "YCbCrSubSampling")]
    pub y_cb_cr_sub_sampling: Option<String>, // e.g., "YCbCr4:2:0 (2 2)"
    #[serde(
        alias = "YResolution",
        deserialize_with = "crate::parse_fn::undef_or_float::float",
        default
    )]
    pub y_resolution: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct LocationMetadata {
    #[serde(alias = "GPSAltitude")]
    pub gps_altitude: Option<String>, // String due to unit/ref ("m Above Sea Level")
    #[serde(alias = "GPSAltitudeRef")]
    pub gps_altitude_ref: Option<String>,
    #[serde(alias = "GPSCoordinates")]
    pub gps_coordinates: Option<String>, // Combined Lat/Lon string
    #[serde(alias = "GPSDOP")]
    pub gps_dop: Option<f64>, // GPS Degree of Precision
    #[serde(
        alias = "GPSDateStamp",
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )] // YYYY:MM:DD
    pub gps_date_stamp: Option<MaybeDateTime>,
    #[serde(
        alias = "GPSDateTime",
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )] // Includes Z
    pub gps_date_time: Option<MaybeDateTime>,
    #[serde(alias = "GPSImgDirection")]
    pub gps_img_direction: Option<f64>,
    #[serde(alias = "GPSImgDirectionRef")]
    pub gps_img_direction_ref: Option<String>,
    #[serde(alias = "GPSLatitude")]
    pub gps_latitude: Option<String>, // String format deg ' " N/S
    #[serde(alias = "GPSLatitudeRef")]
    pub gps_latitude_ref: Option<String>,
    #[serde(alias = "GPSLongitude")]
    pub gps_longitude: Option<String>, // String format deg ' " E/W
    #[serde(alias = "GPSLongitudeRef")]
    pub gps_longitude_ref: Option<String>,
    #[serde(alias = "GPSPosition")]
    pub gps_position: Option<String>, // Combined Lat/Lon string (often same as GPSCoordinates)
    #[serde(
        alias = "GPSProcessingMethod",
        deserialize_with = "crate::parse_fn::string::string",
        default
    )]
    pub gps_processing_method: Option<String>, // e.g., "fused", "GPS", "NETWORK"
    #[serde(
        alias = "GPSTimeStamp",
        deserialize_with = "crate::parse_fn::time::timestamp",
        default
    )] // HH:MM:SS
    pub gps_time_stamp: Option<NaiveTime>,
    #[serde(alias = "GPSVersionID")]
    pub gps_version_id: Option<String>, // e.g., "2.2.0.0"
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct OtherMetadata {
    #[serde(alias = "AIScene")]
    pub ai_scene: Option<i32>, // Assuming integer ID
    pub android_capture_fps: Option<u32>,
    pub android_make: Option<String>,
    pub android_model: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub android_version: Option<String>, // String to handle "7.1.2" etc.
    pub application_record_version: Option<u32>,
    #[serde(alias = "CodedCharacterSet")]
    pub coded_character_set: Option<String>, // e.g., "UTF8"
    pub directory: Option<String>,
    pub envelope_record_version: Option<u32>,
    pub file_name: Option<String>,
    pub file_permissions: Option<String>, // e.g., "-rw-rw-rw-"
    pub file_size: Option<String>,        // String due to unit "kB", "MB"
    pub file_type: Option<String>,        // e.g., "JPEG", "MP4"
    pub file_type_extension: Option<String>, // e.g., "jpg", "mp4"
    #[serde(alias = "FilterId")]
    pub filter_id: Option<u32>,
    pub has_extended_xmp: Option<String>, // Hex string (UUID-like)
    pub hdr: Option<String>,              // e.g., "normal"
    #[serde(alias = "MIMEType")]
    pub mime_type: Option<String>,
    #[serde(alias = "MetaFormat")]
    pub meta_format: Option<String>, // e.g., "mett"
    #[serde(alias = "MetaType")]
    pub meta_type: Option<String>, // e.g., "application/meta"
    pub mirror: Option<bool>,
    #[serde(alias = "OpMode")]
    pub op_mode: Option<u32>,
    pub sensor_type: Option<String>, // e.g., "rear", "front"
    pub zoom_multiple: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct PreviewMetadata {
    pub confidence_map_image: Option<String>,

    pub depth_map_image: Option<String>,

    pub gain_map_image: Option<String>,
    #[serde(alias = "MPImage2")]
    pub mp_image2: Option<String>,

    pub original_image: Option<String>,

    pub thumbnail_image: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct TimeMetadata {
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub create_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub date_created: Option<MaybeDateTime>, // Seems redundant with CreateDate
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub date_time_created: Option<MaybeDateTime>, // Includes timezone
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub date_time_original: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub file_access_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub file_create_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub file_modify_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    // Example: "2015:07:11 11:37:41.746Z"
    pub first_photo_date: Option<MaybeDateTime>, // Or MaybeDateTime if Z is not always there
    #[serde(
        alias = "GPSDateStamp",
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )] // YYYY:MM:DD
    pub gps_date_stamp: Option<MaybeDateTime>, // Duplicated in Location, keep consistent
    #[serde(
        alias = "GPSDateTime",
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )] // Includes Z
    pub gps_date_time: Option<MaybeDateTime>, // Duplicated in Location
    #[serde(
        alias = "GPSTimeStamp",
        deserialize_with = "crate::parse_fn::time::timestamp",
        default
    )] // HH:MM:SS
    pub gps_time_stamp: Option<NaiveTime>, // Duplicated in Location
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    // Example: "2015:07:11 11:38:14.223Z"
    pub last_photo_date: Option<MaybeDateTime>, // Or MaybeDateTime
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub media_create_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub media_modify_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub modify_date: Option<MaybeDateTime>,
    pub offset_time: Option<String>, // e.g., "+02:00"
    pub offset_time_digitized: Option<String>,
    pub offset_time_original: Option<String>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub profile_date_time: Option<MaybeDateTime>,
    // SubSec fields often duplicate the main date but add precision.
    // Assuming your naive parser handles ".ffffff" suffix.
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub sub_sec_create_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub sub_sec_date_time_original: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub sub_sec_modify_date: Option<MaybeDateTime>,
    // SubSecTime appears to be just the fractional part as a string/number
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub sub_sec_time: Option<String>, // Keep as string, parsing requires care
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub sub_sec_time_digitized: Option<String>,
    #[serde(deserialize_with = "crate::parse_fn::string::string", default)]
    pub sub_sec_time_original: Option<String>,
    pub time_created: Option<String>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub track_create_date: Option<MaybeDateTime>,
    #[serde(
        deserialize_with = "crate::parse_fn::datetime::guess_datetime",
        default
    )]
    pub track_modify_date: Option<MaybeDateTime>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct UnknownMetadata {
    // Fields specifically under the "Unknown" key
    #[serde(alias = "CameraId")]
    pub camera_id: Option<u32>,
    pub camera_mode: Option<String>,  // e.g., "AUTO_VIDEO_MODE"
    pub capture_mode: Option<String>, // e.g., "Photo"
    pub is_hdr_active: Option<bool>,
    pub is_night_mode_active: Option<bool>,
    pub lens_facing: Option<String>, // e.g., "Back"
    pub model: Option<String>,
    pub scene: Option<String>, // e.g., "AutoHDR"
    // These look like string representations of arrays
    pub scene_detect_result_confidences: Option<String>,
    pub scene_detect_result_ids: Option<String>,
    pub software: Option<String>,
    pub stable_option: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct VideoMetadata {
    pub avg_bitrate: Option<String>, // String due to unit "Mbps"
    pub color_primaries: Option<String>,
    pub color_profiles: Option<String>, // e.g. "nclx"
    #[serde(
        deserialize_with = "crate::parse_fn::string_list::string_list",
        default
    )]
    pub compatible_brands: Option<Vec<String>>, // e.g. ["isom", "mp42"]
    pub current_time: Option<String>,   // String due to unit "s"
    pub duration: Option<String>,       // String due to unit "s" or format "0:02:26"
    pub graphics_mode: Option<String>,  // e.g., "srcCopy"
    pub handler_description: Option<String>, // e.g. "SoundHandle"
    pub handler_type: Option<String>,
    pub image_height: Option<u32>,
    pub image_width: Option<u32>,
    pub major_brand: Option<String>, // e.g. "MP4 v2 [ISO 14496-14]"
    pub matrix_coefficients: Option<String>,
    pub matrix_structure: Option<String>, // e.g., "1 0 0 0 1 0 0 0 1"
    pub media_data_offset: Option<u64>,
    pub media_data_size: Option<u64>,
    pub media_duration: Option<String>, // String like Duration
    pub media_header_version: Option<u32>,
    pub media_time_scale: Option<u32>,
    pub minor_version: Option<String>, // e.g., "0.0.0"

    pub motion_photo_video: Option<String>,
    pub movie_header_version: Option<u32>,
    pub next_track_id: Option<u32>,
    #[serde(alias = "OpColor")]
    pub op_color: Option<String>, // e.g., "0 0 0"
    pub poster_time: Option<String>,        // String due to unit "s"
    pub preferred_rate: Option<f64>,        // Often 1.0 or 1
    pub preferred_volume: Option<String>,   // String due to "%"
    pub preview_duration: Option<String>,   // String due to unit "s"
    pub preview_time: Option<String>,       // String due to unit "s"
    pub rotation: Option<i32>,              // e.g., 0, 90, 270
    pub selection_duration: Option<String>, // String due to unit "s"
    pub selection_time: Option<String>,     // String due to unit "s"
    pub time_scale: Option<u32>,
    pub track_duration: Option<String>, // String like Duration
    pub track_header_version: Option<u32>,
    #[serde(alias = "TrackID")]
    pub track_id: Option<u32>,
    pub track_layer: Option<i32>,     // Can be negative?
    pub track_volume: Option<String>, // String due to "%"
    pub transfer_characteristics: Option<String>,
    pub video_frame_rate: Option<f64>,
    pub video_full_range_flag: Option<String>, // Full, Limited
}
