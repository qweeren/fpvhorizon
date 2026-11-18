/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.brigadier.CommandDispatcher
 *  net.minecraft.commands.CommandSourceStack
 *  net.minecraft.core.BlockPos
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.world.InteractionHand
 *  net.minecraft.world.InteractionResult
 *  net.minecraft.world.entity.Entity
 *  net.minecraft.world.entity.player.Player
 *  net.minecraft.world.item.Items
 *  net.minecraft.world.level.Level
 *  net.minecraft.world.level.LevelAccessor
 *  net.minecraft.world.level.block.Blocks
 *  net.minecraft.world.level.block.state.BlockState
 *  net.minecraftforge.event.RegisterCommandsEvent
 *  net.minecraftforge.event.TickEvent$PlayerTickEvent$Post
 *  net.minecraftforge.event.entity.player.PlayerEvent$PlayerLoggedOutEvent
 *  net.minecraftforge.event.entity.player.PlayerInteractEvent$RightClickBlock
 *  net.minecraftforge.event.level.BlockEvent$BreakEvent
 *  net.minecraftforge.event.level.BlockEvent$EntityMultiPlaceEvent
 *  net.minecraftforge.event.level.BlockEvent$EntityPlaceEvent
 *  net.minecraftforge.event.level.ExplosionEvent$Detonate
 *  net.minecraftforge.event.server.ServerStartingEvent
 *  net.minecraftforge.event.server.ServerStoppedEvent
 *  net.minecraftforge.fml.loading.FMLPaths
 *  org.slf4j.Logger
 */
package com.taylorgoolsby.fpvdrone.race;

import com.mojang.brigadier.CommandDispatcher;
import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import com.taylorgoolsby.fpvdrone.race.RaceBuild;
import com.taylorgoolsby.fpvdrone.race.RaceCommand;
import com.taylorgoolsby.fpvdrone.race.RaceNavigate;
import com.taylorgoolsby.fpvdrone.race.RaceServer;
import com.taylorgoolsby.fpvdrone.race.RaceWaypoints;
import com.taylorgoolsby.fpvdrone.race.db.Database;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelAccessor;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraftforge.event.RegisterCommandsEvent;
import net.minecraftforge.event.TickEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.event.entity.player.PlayerInteractEvent;
import net.minecraftforge.event.level.BlockEvent;
import net.minecraftforge.event.level.ExplosionEvent;
import net.minecraftforge.event.server.ServerStartingEvent;
import net.minecraftforge.event.server.ServerStoppedEvent;
import net.minecraftforge.fml.loading.FMLPaths;
import org.slf4j.Logger;

public final class RaceEvents {
    private static final long DEBOUNCE_MILLIS = 200L;
    private static final Map<String, Long> lastRightClick = new HashMap<String, Long>();

    private RaceEvents() {
    }

    public static void register() {
        ServerStartingEvent.BUS.addListener(RaceEvents::onServerStarting);
        ServerStoppedEvent.BUS.addListener(RaceEvents::onServerStopped);
        RegisterCommandsEvent.BUS.addListener(RaceEvents::onRegisterCommands);
        BlockEvent.EntityPlaceEvent.BUS.addListener(RaceEvents::onBlockPlace);
        BlockEvent.EntityMultiPlaceEvent.BUS.addListener(RaceEvents::onBlockPlaceMulti);
        BlockEvent.BreakEvent.BUS.addListener(RaceEvents::onBlockBreak);
        ExplosionEvent.Detonate.BUS.addListener(RaceEvents::onExplosion);
        PlayerInteractEvent.RightClickBlock.BUS.addListener(event -> RaceEvents.onRightClickBlock(event));
        TickEvent.PlayerTickEvent.Post.BUS.addListener(RaceEvents::onPlayerTick);
        PlayerEvent.PlayerLoggedOutEvent.BUS.addListener(RaceEvents::onPlayerLoggedOut);
    }

    private static void onServerStarting(ServerStartingEvent event) {
        RaceServer.server = event.getServer();
        Database.setLoggers(arg_0 -> ((Logger)FpvDroneMod.LOGGER).info(arg_0));
        try {
            Database.init(FMLPaths.GAMEDIR.get());
            RaceServer.databaseReady = true;
            RaceServer.databaseError = "";
        }
        catch (Throwable t) {
            RaceServer.databaseReady = false;
            RaceServer.databaseError = "(" + t.getMessage() + ") See latest.log.";
            FpvDroneMod.LOGGER.error("Racing is disabled: the race database could not be prepared.", t);
        }
    }

    private static void onServerStopped(ServerStoppedEvent event) {
        RaceServer.databaseReady = false;
        RaceServer.server = null;
        Database.shutdown();
        RaceBuild.clear();
        RaceNavigate.clear();
        RaceWaypoints.clear();
        lastRightClick.clear();
    }

    private static void onRegisterCommands(RegisterCommandsEvent event) {
        RaceCommand.register((CommandDispatcher<CommandSourceStack>)event.getDispatcher());
    }

    private static boolean isTargetBlock(BlockState state) {
        return state.is((Object)Blocks.TARGET);
    }

    private static void onBlockPlace(BlockEvent.EntityPlaceEvent event) {
        String string;
        if (event.getLevel().isClientSide()) {
            return;
        }
        LevelAccessor levelAccessor = event.getLevel();
        if (!(levelAccessor instanceof Level)) {
            return;
        }
        Level level = (Level)levelAccessor;
        Entity entity = event.getEntity();
        if (entity instanceof Player) {
            Player p = (Player)entity;
            string = p.getName().getString();
        } else {
            string = null;
        }
        String who = string;
        RaceEvents.changedBlock(level, event.getPos(), who);
        if (entity instanceof ServerPlayer) {
            ServerPlayer player = (ServerPlayer)entity;
            if (RaceEvents.isTargetBlock(event.getPlacedBlock())) {
                RaceBuild.startTrack(RaceServer.getId(player.getUUID()), RaceServer.dimensionOf(level), event.getPos(), player);
            }
        }
    }

    private static void onBlockPlaceMulti(BlockEvent.EntityMultiPlaceEvent event) {
        String string;
        if (event.getLevel().isClientSide()) {
            return;
        }
        LevelAccessor levelAccessor = event.getLevel();
        if (!(levelAccessor instanceof Level)) {
            return;
        }
        Level level = (Level)levelAccessor;
        Entity entity = event.getEntity();
        if (entity instanceof Player) {
            Player p = (Player)entity;
            string = p.getName().getString();
        } else {
            string = null;
        }
        String who = string;
        RaceEvents.changedBlock(level, event.getPos(), who);
    }

    private static void onBlockBreak(BlockEvent.BreakEvent event) {
        if (event.getLevel().isClientSide()) {
            return;
        }
        LevelAccessor levelAccessor = event.getLevel();
        if (!(levelAccessor instanceof Level)) {
            return;
        }
        Level level = (Level)levelAccessor;
        Player player = event.getPlayer();
        if (!(player instanceof ServerPlayer)) {
            return;
        }
        ServerPlayer player2 = (ServerPlayer)player;
        if (RaceEvents.isTargetBlock(event.getState())) {
            RaceBuild.removeTrack(RaceServer.getId(player2.getUUID()), RaceServer.dimensionOf(level), event.getPos(), player2);
        }
        RaceEvents.changedBlock(level, event.getPos(), player2.getPlainTextName());
    }

    private static void onExplosion(ExplosionEvent.Detonate event) {
        Level level = event.getLevel();
        if (level.isClientSide()) {
            return;
        }
        if (event.getAffectedBlocks().isEmpty()) {
            return;
        }
        String dimension = RaceServer.dimensionOf(level);
        for (BlockPos pos : event.getAffectedBlocks()) {
            RaceNavigate.changedGateBlock(dimension, pos, "A gate was changed by an explosion");
        }
    }

    private static void changedBlock(Level level, BlockPos pos, String who) {
        Object reason = who == null ? "A gate was changed" : "A gate was changed by " + who;
        RaceNavigate.changedGateBlock(RaceServer.dimensionOf(level), pos, (String)reason);
    }

    private static boolean onRightClickBlock(PlayerInteractEvent.RightClickBlock event) {
        if (event.getLevel().isClientSide()) {
            return false;
        }
        if (event.getHand() != InteractionHand.MAIN_HAND) {
            return false;
        }
        Player player = event.getEntity();
        if (!(player instanceof ServerPlayer)) {
            return false;
        }
        ServerPlayer player2 = (ServerPlayer)player;
        Level level = event.getLevel();
        BlockPos pos = event.getPos();
        String userId = RaceServer.getId(player2.getUUID());
        String dimension = RaceServer.dimensionOf(level);
        boolean isStick = player2.getMainHandItem().is((Object)Items.STICK);
        boolean building = RaceBuild.checkBuildingMode(userId);
        boolean isTarget = RaceEvents.isTargetBlock(level.getBlockState(pos));
        boolean emptyHand = player2.getMainHandItem().isEmpty();
        if (isStick && building) {
            if (!RaceEvents.debounce(userId, dimension, pos)) {
                return true;
            }
            RaceBuild.addBlock(userId, dimension, pos, event.getFace(), RaceServer.solidChecker(level), player2);
            event.setCancellationResult((InteractionResult)InteractionResult.SUCCESS);
            return true;
        }
        if (isTarget && emptyHand && !building) {
            if (!RaceEvents.debounce(userId, dimension, pos)) {
                return false;
            }
            RaceNavigate.handleRightClickTrack(userId, dimension, pos, RaceServer.solidChecker(level), player2);
        }
        return false;
    }

    private static boolean debounce(String userId, String dimension, BlockPos pos) {
        long now = System.currentTimeMillis();
        if (lastRightClick.size() > 256) {
            lastRightClick.values().removeIf(t -> now - t > 5000L);
        }
        String key = userId + "|" + dimension + "|" + pos.asLong();
        Long last = lastRightClick.get(key);
        lastRightClick.put(key, now);
        return last == null || now - last >= 200L;
    }

    private static void onPlayerTick(TickEvent.PlayerTickEvent.Post event) {
        Player player = event.player();
        if (!(player instanceof ServerPlayer)) {
            return;
        }
        ServerPlayer player2 = (ServerPlayer)player;
        String userId = RaceServer.getId(player2.getUUID());
        if (!RaceNavigate.checkRacingMode(userId)) {
            return;
        }
        RaceNavigate.onPlayerMoved(userId, RaceServer.dimensionOf((Level)player2.level()), new double[]{player2.getX(), player2.getY(), player2.getZ()}, player2);
    }

    private static void onPlayerLoggedOut(PlayerEvent.PlayerLoggedOutEvent event) {
        Player player = event.getEntity();
        if (!(player instanceof ServerPlayer)) {
            return;
        }
        ServerPlayer player2 = (ServerPlayer)player;
        UUID uuid = player2.getUUID();
        String userId = RaceServer.getId(uuid);
        RaceBuild.exitBuildingMode(userId, null);
        RaceNavigate.exitRacingMode(userId, null, null);
        RaceWaypoints.forget(uuid);
    }
}
