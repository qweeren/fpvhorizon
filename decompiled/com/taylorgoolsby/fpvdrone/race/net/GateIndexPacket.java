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

public record GateIndexPacket(int nextGate) {
    public static final StreamCodec<RegistryFriendlyByteBuf, GateIndexPacket> CODEC = StreamCodec.of((buf, msg) -> buf.writeVarInt(msg.nextGate()), buf -> new GateIndexPacket(buf.readVarInt()));

    public static void handle(GateIndexPacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> RaceClientState.setNextGate(msg.nextGate()));
        ctx.setPacketHandled(true);
    }
}
