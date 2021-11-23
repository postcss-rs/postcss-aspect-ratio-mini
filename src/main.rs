use plugin_aspect_ratio_mini::{AspectRatioMini, SimplePrettier};
use recursive_parser::{
    parser::Parser,
    visitor::{Visit, VisitMut},
};
use std::fmt::format;

fn main() {
    let css = r#"
    
    
    .aspect-box {
    position: relative;
}

[aspectratio][aspect-ratio="16:9"] {
    aspect-ratio: '16:9';
}

[aspectratio][aspect-ratio="16/9"] {
    aspect-ratio: 16 / 9;
}

    "#;
    let parser = Parser::new(css);
    let mut root = parser.parse();
    let result = AspectRatioMini::transform(&mut root, 4);
    println!("{}", result);
}
