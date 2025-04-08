use crate::core::exiftool::ExifTool;
use crate::error::ExifToolError;
use serde_json::Value;

impl ExifTool {
    /// Execute any command and get the result as a list of strings (lines).
    /// The command executed by this function is as follows:
    ///
    /// `exiftool {...args}`
    pub fn execute_str(&mut self, cmd_args: &[&str]) -> Result<Vec<String>, ExifToolError> {
        let raw_output = self.execute_raw(cmd_args)?;
        let lines: Vec<String> = String::from_utf8_lossy(&raw_output)
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(lines)
    }

    /// Execute any command and get the result in JSON form.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {...args}`
    ///
    /// The output will be a json array of objects, each object describing one input file.
    /// You can pass as many files as you want to this.
    ///
    /// For example:
    /// ```rs
    /// let files = vec![file1, file2, file3];
    /// let value = execute_json(files)?;
    /// ```
    ///
    /// You can tell exiftool to structure the output by grouping into categories with `-g1` or `-g2`.
    pub fn execute_json(&mut self, args: &[&str]) -> Result<Value, ExifToolError> {
        let mut cmd_args = vec!["-json"];
        cmd_args.extend_from_slice(args);
        let output_bytes = self.execute_raw(&cmd_args)?;
        let value: Value = serde_json::from_slice(&output_bytes)?;
        Ok(value)
    }
}
