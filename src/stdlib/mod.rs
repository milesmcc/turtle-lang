pub fn get_std_resource(path: &str) -> Option<String> {
    match path {
        "@prelude" => Some(include_str!("prelude.lisp").to_string()),
        "@map" => Some(include_str!("map.lisp").to_string()),
        _ => None,
    }
}
