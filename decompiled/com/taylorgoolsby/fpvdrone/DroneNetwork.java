/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.resources.Identifier
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraftforge.network.ChannelBuilder
 *  net.minecraftforge.network.PacketDistributor
 *  net.minecraftforge.network.SimpleChannel
 *  net.minecraftforge.network.simple.SimpleProtocol
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneOrientationBroadcastPacket;
import com.taylorgoolsby.fpvdrone.DroneOrientationPacket;
import com.taylorgoolsby.fpvdrone.DroneStatePacket;
import com.taylorgoolsby.fpvdrone.DroneTogglePacket;
import com.taylorgoolsby.fpvdrone.race.net.BuildGatePacket;
import com.taylorgoolsby.fpvdrone.race.net.GateIndexPacket;
import com.taylorgoolsby.fpvdrone.race.net.LapStartPacket;
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardPacket;
import com.taylorgoolsby.fpvdrone.race.net.RaceModePacket;
import com.taylorgoolsby.fpvdrone.race.net.SetBuildModePacket;
import java.util.UUID;
import net.minecraft.resources.Identifier;
import net.minecraft.server.level.ServerPlayer;
import net.minecraftforge.network.ChannelBuilder;
import net.minecraftforge.network.PacketDistributor;
import net.minecraftforge.network.SimpleChannel;
import net.minecraftforge.network.simple.SimpleProtocol;

public final class DroneNetwork {
    private static final int PROTOCOL_VERSION = 3;
    public static final SimpleChannel CHANNEL = ChannelBuilder.named((Identifier)Identifier.fromNamespaceAndPath((String)"fpvdrone", (String)"main")).networkProtocolVersion(3).simpleChannel();

    private DroneNetwork() {
    }

    public static void register() {
        ((SimpleProtocol)CHANNEL.play().serverbound(flow -> flow.add(DroneTogglePacket.class, DroneTogglePacket.CODEC, DroneTogglePacket::handle).add(DroneOrientationPacket.class, DroneOrientationPacket.CODEC, DroneOrientationPacket::handle))).clientbound(flow -> flow.add(DroneStatePacket.class, DroneStatePacket.CODEC, DroneStatePacket::handle).add(DroneOrientationBroadcastPacket.class, DroneOrientationBroadcastPacket.CODEC, DroneOrientationBroadcastPacket::handle).add(SetBuildModePacket.class, SetBuildModePacket.CODEC, SetBuildModePacket::handle).add(BuildGatePacket.class, BuildGatePacket.CODEC, BuildGatePacket::handle).add(RaceModePacket.class, RaceModePacket.CODEC, RaceModePacket::handle).add(GateIndexPacket.class, GateIndexPacket.CODEC, GateIndexPacket::handle).add(LapStartPacket.class, LapStartPacket.CODEC, LapStartPacket::handle).add(LeaderboardPacket.class, LeaderboardPacket.CODEC, LeaderboardPacket::handle));
    }

    public static void sendToggle(boolean active) {
        CHANNEL.send((Object)new DroneTogglePacket(active), PacketDistributor.SERVER.noArg());
    }

    public static void sendOrientation(float x, float y, float z, float w) {
        CHANNEL.send((Object)new DroneOrientationPacket(x, y, z, w), PacketDistributor.SERVER.noArg());
    }

    public static void broadcastState(UUID player, boolean active) {
        CHANNEL.send((Object)new DroneStatePacket(player, active), PacketDistributor.ALL.noArg());
    }

    public static void sendStateTo(ServerPlayer target, UUID player, boolean active) {
        CHANNEL.send((Object)new DroneStatePacket(player, active), PacketDistributor.PLAYER.with((Object)target));
    }

    public static void relayOrientation(ServerPlayer pilot, float x, float y, float z, float w) {
        CHANNEL.send((Object)new DroneOrientationBroadcastPacket(pilot.getUUID(), x, y, z, w), PacketDistributor.TRACKING_ENTITY.with((Object)pilot));
    }
}
