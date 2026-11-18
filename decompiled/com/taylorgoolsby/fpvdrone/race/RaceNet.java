/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraftforge.network.PacketDistributor
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.race.Ids;
import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.RaceLap;
import com.taylorgoolsby.fpvdrone.race.RaceTrack;
import com.taylorgoolsby.fpvdrone.race.net.BuildGatePacket;
import com.taylorgoolsby.fpvdrone.race.net.GateBox;
import com.taylorgoolsby.fpvdrone.race.net.GateIndexPacket;
import com.taylorgoolsby.fpvdrone.race.net.LapStartPacket;
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardEntry;
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardPacket;
import com.taylorgoolsby.fpvdrone.race.net.RaceModePacket;
import com.taylorgoolsby.fpvdrone.race.net.SetBuildModePacket;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.server.level.ServerPlayer;
import net.minecraftforge.network.PacketDistributor;

public final class RaceNet {
    private RaceNet() {
    }

    private static void send(ServerPlayer player, Object packet) {
        if (player == null) {
            return;
        }
        DroneNetwork.CHANNEL.send(packet, PacketDistributor.PLAYER.with((Object)player));
    }

    public static GateBox box(RaceGate gate) {
        return new GateBox(gate.origin.getX(), gate.origin.getY(), gate.origin.getZ(), gate.farthest.getX(), gate.farthest.getY(), gate.farthest.getZ());
    }

    public static void sendBuildMode(ServerPlayer player, boolean active) {
        RaceNet.send(player, new SetBuildModePacket(active));
    }

    public static void sendBuildGate(ServerPlayer player, RaceGate gate) {
        RaceNet.send(player, new BuildGatePacket(gate.dimension, RaceNet.box(gate)));
    }

    public static void sendRaceStart(ServerPlayer player, RaceTrack track) {
        ArrayList<GateBox> boxes = new ArrayList<GateBox>(track.gates.size());
        for (RaceGate gate : track.gates) {
            boxes.add(RaceNet.box(gate));
        }
        RaceNet.send(player, new RaceModePacket(true, track.name, track.dimension, boxes));
    }

    public static void sendRaceStop(ServerPlayer player) {
        RaceNet.send(player, RaceModePacket.stop());
    }

    public static void sendNextGate(ServerPlayer player, int index) {
        RaceNet.send(player, new GateIndexPacket(index));
    }

    public static void sendLapStart(ServerPlayer player, int lastLapMillis) {
        RaceNet.send(player, new LapStartPacket(lastLapMillis));
    }

    public static void sendLeaderboard(ServerPlayer player, List<RaceLap> laps) {
        ArrayList<LeaderboardEntry> entries = new ArrayList<LeaderboardEntry>(laps.size());
        for (RaceLap lap : laps) {
            entries.add(new LeaderboardEntry(Ids.fromHex(lap.userId), lap.userName, lap.millis));
        }
        RaceNet.send(player, new LeaderboardPacket(entries));
    }
}
