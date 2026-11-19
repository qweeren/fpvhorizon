fn fpvdrone_pad(down: bool) -> ::core::ops::ControlFlow<Result<()>> {
    if down && mod_fpvdrone::is_flying() {
        ::core::ops::ControlFlow::Break(Ok(()))
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
