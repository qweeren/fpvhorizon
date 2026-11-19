fn fpvhorizon_turn(xo: f64, yo: f64) -> ::core::ops::ControlFlow<()> {
    if mod_fpvhorizon::take_turn(xo, yo) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
