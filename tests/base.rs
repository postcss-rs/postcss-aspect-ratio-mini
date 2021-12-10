#[cfg(test)]
mod test_basic {
    use super::*;
    use std::fs::read_to_string;

    use plugin_aspect_ratio_mini::AspectRatioMini;
    use recursive_parser::parser::Parser;

    #[test]
    fn test_() -> std::io::Result<()> {
        let prefix_list = ["damo", "default", "double-quote", "precision"];

        for prefix in prefix_list {
            let file_path = format!("./tests/fixtures/{}.css", prefix);
            let expected_file_path = format!("./tests/fixtures/{}.expected.css", prefix);
            let file = read_to_string(file_path)?;
            let expected_file = convert_line_feed(read_to_string(expected_file_path)?);
            let parser = Parser::new(&file);
            let mut root = parser.parse();

            let result = AspectRatioMini::transform(&mut root, 4);
            similar_asserts::assert_str_eq!(result, expected_file);
        }
        Ok(())
    }
}
fn convert_line_feed(input: String) -> String {
    if cfg!(target_os = "windows") {
        newline_converter::unix2dos(&input).to_string()
    } else {
        newline_converter::dos2unix(&input).to_string()
    }
}
