// native_client_packet hook: handles incoming play packets.
fn fpvhorizon_packet(
    game: &mut crate::game::Game<'_>,
    packet: &horizon_client::mod_api::ClientboundPlayPacket,
) {
    crate::game::fpvhorizon_adapter::packet(game, packet);
}
