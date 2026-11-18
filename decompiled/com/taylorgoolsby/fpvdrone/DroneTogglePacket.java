/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.world.entity.player.Abilities
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Abilities;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record DroneTogglePacket(boolean active) {
    public static final StreamCodec<RegistryFriendlyByteBuf, DroneTogglePacket> CODEC = StreamCodec.of((buf, msg) -> buf.writeBoolean(msg.active()), buf -> new DroneTogglePacket(buf.readBoolean()));
    private static final Map<UUID, boolean[]> PREV_ABILITIES = new ConcurrentHashMap<UUID, boolean[]>();

    public static void handle(DroneTogglePacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> {
            ServerPlayer sender = ctx.getSender();
            if (sender == null) {
                return;
            }
            boolean active = msg.active() && sender.isAlive();
            DroneRegistry.set(sender.getUUID(), active);
            DroneTogglePacket.applyFlightExemption(sender, active);
            sender.refreshDimensions();
            DroneNetwork.broadcastState(sender.getUUID(), active);
            FpvDroneMod.LOGGER.info("[fpvdrone] toggle from {}: requested={} applied={} serverBoxWidth={} serverBoxHeight={}", new Object[]{sender.getPlainTextName(), msg.active(), active, Float.valueOf(sender.getBbWidth()), Float.valueOf(sender.getBbHeight())});
        });
        ctx.setPacketHandled(true);
    }

    private static void applyFlightExemption(ServerPlayer player, boolean active) {
        UUID id = player.getUUID();
        Abilities abilities = player.getAbilities();
        if (active) {
            PREV_ABILITIES.putIfAbsent(id, new boolean[]{abilities.mayfly, abilities.flying});
            abilities.mayfly = true;
            abilities.flying = true;
            player.onUpdateAbilities();
        } else {
            boolean[] prev = PREV_ABILITIES.remove(id);
            if (prev != null) {
                abilities.mayfly = prev[0];
                abilities.flying = prev[0] && prev[1];
                player.onUpdateAbilities();
            }
        }
    }

    public static void maintainFlight(ServerPlayer player) {
        if (!DroneRegistry.isDrone(player.getUUID())) {
            return;
        }
        Abilities abilities = player.getAbilities();
        if (!(player.onGround() || abilities.flying && abilities.mayfly)) {
            abilities.mayfly = true;
            abilities.flying = true;
            player.onUpdateAbilities();
        }
    }

    public static void forget(UUID id) {
        PREV_ABILITIES.remove(id);
    }
}
