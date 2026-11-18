/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone.race.net;

import com.taylorgoolsby.fpvdrone.race.RaceClientState;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record SetBuildModePacket(boolean active) {
    public static final StreamCodec<RegistryFriendlyByteBuf, SetBuildModePacket> CODEC = StreamCodec.of((buf, msg) -> buf.writeBoolean(msg.active()), buf -> new SetBuildModePacket(buf.readBoolean()));

    public static void handle(SetBuildModePacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> RaceClientState.setBuildMode(msg.active()));
        ctx.setPacketHandled(true);
    }
}
