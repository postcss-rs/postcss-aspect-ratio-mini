use plugin_aspect_ratio_mini::AspectRatioMini;
use recursive_parser::{
    parser::Parser,
};
use std::time::Instant;

fn main() {
    let _css = r#"
    
    
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
    let bootstrap = include_str!("../assets/bootstrap.css");
    let start = Instant::now();
    let parser = Parser::new(bootstrap);
    let mut root = parser.parse();
    let _result = AspectRatioMini::transform(&mut root, 4);
    // println!("{}", result);
    println!("{:?}", start.elapsed());
}
