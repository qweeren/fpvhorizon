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
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardEntry;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record LeaderboardPacket(List<LeaderboardEntry> entries) {
    public static final StreamCodec<RegistryFriendlyByteBuf, LeaderboardPacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeVarInt(msg.entries().size());
        for (LeaderboardEntry e : msg.entries()) {
            buf.writeUUID(e.userId());
            buf.writeUtf(e.name());
            buf.writeInt(e.millis());
        }
    }, buf -> {
        int n = buf.readVarInt();
        ArrayList<LeaderboardEntry> entries = new ArrayList<LeaderboardEntry>(n);
        for (int i = 0; i < n; ++i) {
            entries.add(new LeaderboardEntry(buf.readUUID(), buf.readUtf(), buf.readInt()));
        }
        return new LeaderboardPacket(entries);
    });

    public static void handle(LeaderboardPacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> RaceClientState.setLeaderboard(msg.entries()));
        ctx.setPacketHandled(true);
    }
}
