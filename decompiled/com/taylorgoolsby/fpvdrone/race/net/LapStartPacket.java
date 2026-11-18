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

public record LapStartPacket(int lastLapMillis) {
    public static final StreamCodec<RegistryFriendlyByteBuf, LapStartPacket> CODEC = StreamCodec.of((buf, msg) -> buf.writeInt(msg.lastLapMillis()), buf -> new LapStartPacket(buf.readInt()));

    public static void handle(LapStartPacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> RaceClientState.startLap(msg.lastLapMillis()));
        ctx.setPacketHandled(true);
    }
}
