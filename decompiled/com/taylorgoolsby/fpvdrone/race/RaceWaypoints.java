/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.Vec3i
 *  net.minecraft.network.protocol.Packet
 *  net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.world.waypoints.Waypoint$Icon
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.V3;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import net.minecraft.core.Vec3i;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.waypoints.Waypoint;

public final class RaceWaypoints {
    private static final int COLOR = 16765286;
    private static final Map<UUID, UUID> WAYPOINT_IDS = new HashMap<UUID, UUID>();

    private RaceWaypoints() {
    }

    public static void show(ServerPlayer player, RaceGate gate) {
        V3 c = gate.getCenter();
        Vec3i pos = new Vec3i(Math.round(c.x), Math.round(c.y), Math.round(c.z));
        UUID id = WAYPOINT_IDS.computeIfAbsent(player.getUUID(), k -> UUID.randomUUID());
        Waypoint.Icon icon = new Waypoint.Icon();
        icon.color = Optional.of(16765286);
        player.connection.send((Packet)ClientboundTrackedWaypointPacket.addWaypointPosition((UUID)id, (Waypoint.Icon)icon, (Vec3i)pos));
    }

    public static void hide(ServerPlayer player) {
        if (player == null) {
            return;
        }
        UUID id = WAYPOINT_IDS.remove(player.getUUID());
        if (id == null) {
            return;
        }
        player.connection.send((Packet)ClientboundTrackedWaypointPacket.removeWaypoint((UUID)id));
    }

    public static void forget(UUID playerUuid) {
        WAYPOINT_IDS.remove(playerUuid);
    }

    public static void clear() {
        WAYPOINT_IDS.clear();
    }
}
