use serde::Deserialize;
pub type ExifOutput = Vec<ExifData>;
#[derive(Debug, Deserialize)]
pub struct ExifData {
    #[serde(rename = "Aperture")]
    pub aperture: Option<f64>,

    #[serde(rename = "ApertureValue")]
    pub aperture_value: Option<f64>,

    #[serde(rename = "BitsPerSample")]
    pub bits_per_sample: Option<u32>,

    #[serde(rename = "BlueMatrixColumn")]
    pub blue_matrix_column: Option<String>,

    #[serde(rename = "BlueTRC")]
    pub blue_trc: Option<String>,

    #[serde(rename = "BrightnessValue")]
    pub brightness_value: Option<f64>,

    #[serde(rename = "CMMFlags")]
    pub cmm_flags: Option<String>,

    #[serde(rename = "ChromaticAdaptation")]
    pub chromatic_adaptation: Option<String>,

    #[serde(rename = "ColorComponents")]
    pub color_components: Option<u32>,

    #[serde(rename = "ColorSpace")]
    pub color_space: Option<String>,

    #[serde(rename = "ColorSpaceData")]
    pub color_space_data: Option<String>,

    #[serde(rename = "ComponentsConfiguration")]
    pub components_configuration: Option<String>,

    #[serde(rename = "Compression")]
    pub compression: Option<String>,

    #[serde(rename = "CreateDate")]
    pub create_date: Option<String>,

    #[serde(rename = "DateTimeOriginal")]
    pub date_time_original: Option<String>,

    #[serde(rename = "DeviceAttributes")]
    pub device_attributes: Option<String>,

    #[serde(rename = "Directory")]
    pub directory: Option<String>,

    #[serde(rename = "ExposureCompensation")]
    pub exposure_compensation: Option<f64>,

    #[serde(rename = "ExposureMode")]
    pub exposure_mode: Option<String>,

    #[serde(rename = "ExposureProgram")]
    pub exposure_program: Option<String>,

    #[serde(rename = "ExposureTime")]
    pub exposure_time: Option<String>,

    #[serde(rename = "FNumber")]
    pub f_number: Option<f64>,

    #[serde(rename = "FileName")]
    pub file_name: Option<String>,

    #[serde(rename = "FileSize")]
    pub file_size: Option<String>,

    #[serde(rename = "FileType")]
    pub file_type: Option<String>,

    #[serde(rename = "FileTypeExtension")]
    pub file_type_extension: Option<String>,

    #[serde(rename = "Flash")]
    pub flash: Option<String>,

    #[serde(rename = "FocalLength")]
    pub focal_length: Option<String>,

    #[serde(rename = "ISO")]
    pub iso: Option<u32>,

    #[serde(rename = "ImageHeight")]
    pub image_height: Option<u32>,

    #[serde(rename = "ImageWidth")]
    pub image_width: Option<u32>,

    #[serde(rename = "Make")]
    pub make: Option<String>,

    #[serde(rename = "Model")]
    pub model: Option<String>,

    #[serde(rename = "Orientation")]
    pub orientation: Option<String>,

    #[serde(rename = "Software")]
    pub software: Option<String>,

    #[serde(rename = "SourceFile")]
    pub source_file: Option<String>,
}