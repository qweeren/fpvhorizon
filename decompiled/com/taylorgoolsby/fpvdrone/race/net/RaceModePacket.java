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
import java.util.ArrayList;
import java.util.List;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record RaceModePacket(boolean racing, String trackName, String dimension, List<GateBox> gates) {
    public static final StreamCodec<RegistryFriendlyByteBuf, RaceModePacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeBoolean(msg.racing());
        buf.writeUtf(msg.trackName());
        buf.writeUtf(msg.dimension());
        buf.writeVarInt(msg.gates().size());
        for (GateBox g : msg.gates()) {
            g.write((FriendlyByteBuf)buf);
        }
    }, buf -> {
        boolean racing = buf.readBoolean();
        String name = buf.readUtf();
        String dimension = buf.readUtf();
        int n = buf.readVarInt();
        ArrayList<GateBox> gates = new ArrayList<GateBox>(n);
        for (int i = 0; i < n; ++i) {
            gates.add(GateBox.read((FriendlyByteBuf)buf));
        }
        return new RaceModePacket(racing, name, dimension, gates);
    });

    public static RaceModePacket stop() {
        return new RaceModePacket(false, "", "", List.of());
    }

    public static void handle(RaceModePacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> {
            if (msg.racing()) {
                RaceClientState.startRacing(msg.trackName(), msg.dimension(), msg.gates());
            } else {
                RaceClientState.stopRacing();
            }
        });
        ctx.setPacketHandled(true);
    }
}
