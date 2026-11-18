/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.FriendlyByteBuf
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone.race.net;

import com.taylorgoolsby.fpvdrone.race.RaceClientState;
import com.taylorgoolsby.fpvdrone.race.net.GateBox;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record BuildGatePacket(String dimension, GateBox gate) {
    public static final StreamCodec<RegistryFriendlyByteBuf, BuildGatePacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeUtf(msg.dimension());
        msg.gate().write((FriendlyByteBuf)buf);
    }, buf -> new BuildGatePacket(buf.readUtf(), GateBox.read((FriendlyByteBuf)buf)));

    public static void handle(BuildGatePacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> RaceClientState.addBuildGate(msg.dimension(), msg.gate()));
        ctx.setPacketHandled(true);
    }
}
