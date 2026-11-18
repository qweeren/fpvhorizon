/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.logging.LogUtils
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.world.entity.player.Player
 *  net.minecraftforge.event.TickEvent$PlayerTickEvent$Post
 *  net.minecraftforge.event.entity.player.PlayerEvent$PlayerLoggedInEvent
 *  net.minecraftforge.event.entity.player.PlayerEvent$PlayerLoggedOutEvent
 *  net.minecraftforge.event.entity.player.PlayerEvent$PlayerRespawnEvent
 *  net.minecraftforge.fml.common.Mod
 *  net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext
 *  net.minecraftforge.fml.loading.FMLEnvironment
 *  org.slf4j.Logger
 */
package com.taylorgoolsby.fpvdrone;

import com.mojang.logging.LogUtils;
import com.taylorgoolsby.fpvdrone.ClientInit;
import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import com.taylorgoolsby.fpvdrone.DroneTogglePacket;
import com.taylorgoolsby.fpvdrone.race.RaceEvents;
import java.util.UUID;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraftforge.event.TickEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;
import net.minecraftforge.fml.loading.FMLEnvironment;
import org.slf4j.Logger;

@Mod(value="fpvdrone")
public final class FpvDroneMod {
    public static final String MODID = "fpvdrone";
    public static final Logger LOGGER = LogUtils.getLogger();
    private static final float DRONE_BOX_HEIGHT = 0.3f;

    public FpvDroneMod(FMLJavaModLoadingContext context) {
        DroneNetwork.register();
        PlayerEvent.PlayerLoggedInEvent.BUS.addListener(FpvDroneMod::onPlayerLoggedIn);
        PlayerEvent.PlayerLoggedOutEvent.BUS.addListener(FpvDroneMod::onPlayerLoggedOut);
        PlayerEvent.PlayerRespawnEvent.BUS.addListener(FpvDroneMod::onPlayerRespawn);
        TickEvent.PlayerTickEvent.Post.BUS.addListener(FpvDroneMod::onPlayerTick);
        RaceEvents.register();
        if (FMLEnvironment.dist.isClient()) {
            ClientInit.init();
        }
        LOGGER.info("FPV Drone mod loaded");
    }

    private static void onPlayerTick(TickEvent.PlayerTickEvent.Post event) {
        Player player = event.player();
        if (player instanceof ServerPlayer) {
            ServerPlayer sp = (ServerPlayer)player;
            DroneTogglePacket.maintainFlight(sp);
        } else if (player.level().isClientSide()) {
            FpvDroneMod.syncRemoteDimensions(player);
        }
    }

    private static void syncRemoteDimensions(Player player) {
        boolean hasDroneBox;
        boolean drone = DroneRegistry.isDrone(player.getUUID());
        boolean bl = hasDroneBox = Math.abs(player.getBbHeight() - 0.3f) < 0.001f;
        if (drone != hasDroneBox) {
            player.refreshDimensions();
        }
    }

    private static void onPlayerLoggedIn(PlayerEvent.PlayerLoggedInEvent event) {
        Player player = event.getEntity();
        if (!(player instanceof ServerPlayer)) {
            return;
        }
        ServerPlayer joining = (ServerPlayer)player;
        for (UUID id : DroneRegistry.snapshot()) {
            DroneNetwork.sendStateTo(joining, id, true);
        }
    }

    private static void onPlayerLoggedOut(PlayerEvent.PlayerLoggedOutEvent event) {
        UUID id = event.getEntity().getUUID();
        DroneRegistry.set(id, false);
        DroneTogglePacket.forget(id);
        if (event.getEntity() instanceof ServerPlayer) {
            DroneNetwork.broadcastState(id, false);
        }
    }

    private static void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {
        UUID id = event.getEntity().getUUID();
        DroneRegistry.set(id, false);
        DroneTogglePacket.forget(id);
        event.getEntity().refreshDimensions();
        if (event.getEntity() instanceof ServerPlayer) {
            DroneNetwork.broadcastState(id, false);
        }
    }
}
