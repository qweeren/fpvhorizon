fn fpvdrone_fisheye(value: Vec<String>) -> Vec<String> {
    let mut value = value;
    // An empty list is the title screen or no world: nothing is drawn through the chains.
    if !value.is_empty() {
        if let Some(effect) = mod_fpvdrone::fisheye_effect() {
            value.push(effect.to_owned());
        }
    }
    value
}
