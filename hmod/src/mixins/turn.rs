fn fpvdrone_turn(xo: f64, yo: f64) -> ::core::ops::ControlFlow<()> {
    if mod_fpvdrone::take_turn(xo, yo) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
