// native_client_packet hook: handles incoming play packets.
fn fpvdrone_packet(
    game: &mut crate::game::Game<'_>,
    packet: &horizon_client::mod_api::ClientboundPlayPacket,
) {
    crate::game::fpvdrone_adapter::packet(game, packet);
}
