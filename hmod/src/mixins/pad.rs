fn fpvhorizon_pad(down: bool) -> ::core::ops::ControlFlow<Result<()>> {
    if down && mod_fpvhorizon::is_flying() {
        ::core::ops::ControlFlow::Break(Ok(()))
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
